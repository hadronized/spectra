// TODO: optimize the allocation scheme

//! Shader module.
//!
//! Shader functions and declarations can be grouped in so-called *modules*. Modules structure is
//! inherently tied to the filesystem’s tree.
//!
//! You’re not supposed to use modules at the Rust level, even though you can. You actually want to
//! write modules that will be used by shader programs using the SPSL language.
//!
//! # SPSL
//!
//! Spectra Shading Language is a superset of [GLSL](https://en.wikipedia.org/wiki/OpenGL_Shading_Language)
//! with extra candies, such as:
//!
//! - module imports/exports;
//! - interface, uniforms, blocks, structs, etc. deduplication
//! - functional programming style
//!
//! ## Define once, use everywhere
//!
//! The idea is that you can refactor the code you use at several places into *modules* – in the
//! same way you do in Rust, and then `import` those modules in other ones.
//!
//! This is achieved with the `from foo.bar.zoo import (yyyy, zzzz)` pattern. You typically want to put
//! that line at the top of your module – you can put several. This will import the `yyyy` and
//! `zzzz` symbols from the `foo.bar.zoo` module. The `(*)` form is called an import list and must
//! contain something.
//!
//! > Note on paths: because of the bijective relation between modules and files, if you import the
//! > `foo.bar.zoo` module, the file `foo/bar/zoo.spsl` must be reachable.
//!
//! > Note on import lists: currently, import lists are just informative. By default, all symbols
//! > are imported. Future plans will restrict them to the one only defined in the import lists.
//!
//! ## Pipeline modules
//!
//! In SPSL, there’s no such thing such as a *stage*. You cannot declare a *vertex shader*, a
//! *geometry shader*, a *fragment shader* or any *tessellation shaders*. Instead, you write
//! pipelines directly.
//!
//! A pipeline is just a special module that contains special functions. Up to now, you can find
//! three functions:
//!
//! | Function name     | Mandatory? | Role                                                              |
//! | -------------     | ---------- | ----                                                              |
//! | `map_vertex`      | yes        | Called on each vertex in the pipeline’s stream                    |
//! | `concat_map_prim` | no         | Called on each primitive generated via the `map_vertex` function  |
//! | `map_fragment`    | yes        | Called on each rasterized fragment                                |
//!
//! ### `map_vertex`
//!
//! This mandatory function must be defined and will be called on each vertex in the input stream.
//! It takes a variable number of arguments and its return type must be provided. Both the arguments
//! and return types form a *contract* that binds the function to the input and output stream. The
//! order of the arguments matters, as it must be the same order as in your tessellation’s buffers.
//!
//! For instance, if you want to process a stream of vertices which have a 3D-floating position and
//! a 4D-floating color and return only the color, you’d something like this:
//!
//! ```glsl
//! struct Vertex {
//!   vec4 gl_Position; // this is mandatory as it will be fetched by the pipeline
//!   vec4 color;
//! };
//!
//! Vertex map_vertex(vec3 position, vec4 color) {
//!   return Vertex(vec4(position, 1.), color);
//! }
//! ```
//!
//! If at some time you come to the realization that you also need the position information in the
//! result, you just have to change the above code to:
//!
//! ```glsl
//! struct Vertex {
//!   vec4 gl_Position; // this is mandatory as it will be fetched by the pipeline
//!   vec3 position;
//!   vec4 color;
//! };
//!
//! Vertex map_vertex(vec3 position, vec4 color) {
//!   return Vertex(vec4(position, 1.), position, color);
//! }
//! ```
//!
//! > Note on the return type: the name of this type is completely up to you. Nothing is enforced,
//! > use the type name you think is the best. `Vertex` is a de facto name because it seems natural
//! > to use it, but if you dislike such a name, feel free to use another.
//!
//! ### `concat_map_prim`
//!
//! This optional function takes an array of vertices which type is the same as `map_vertex`’
//! result’s type and outputs a stream of primitives:
//!
//! ```glsl
//! layout (triangles_strip, max_vertices = 3) struct Prim {
//!   // TODO
//! };
//!
//! void concat_map_prim(Vertex[3] vertices) {
//!   
//! }
//! ```

use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::iter::once;
use std::path::PathBuf;
use glsl::writer;

use render::shader::lang::parser;
// FIXME: qualified use, it’s ugly now
use render::shader::lang::syntax::{Block, Declaration, ExternalDeclaration, FunctionDefinition, FullySpecifiedType,
                                   FunctionParameterDeclaration, FunctionParameterDeclarator, InitDeclaratorList, Expr,
                                   Module as SyntaxModule, SingleDeclaration, StorageQualifier,
                                   StructSpecifier, StructFieldSpecifier, LayoutQualifier, TypeName,
                                   TypeSpecifier, TypeSpecifierNonArray, TypeQualifier, TypeQualifierSpec, LayoutQualifierSpec};
use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};

/// Shader module.
///
/// A shader module is a piece of GLSL code with optional import lists (dependencies).
///
/// You’re not supposed to directly manipulate any object of this type. You just write modules on
/// disk and let everything happen automatically for you.
#[derive(Clone, Debug, PartialEq)]
pub struct Module(pub SyntaxModule); // FIXME: remove the pub

impl Module {
  /// Retrieve all the modules this module depends on, without duplicates.
  pub fn deps(&self, store: &mut Store, key: &ModuleKey) -> Result<Vec<ModuleKey>, DepsError> {
    let mut deps = Vec::new();
    self.deps_no_cycle(store, &key, &mut Vec::new(), &mut deps).map(|_| deps)
  }

  fn deps_no_cycle(&self, store: &mut Store, key: &ModuleKey, parents: &mut Vec<ModuleKey>, deps: &mut Vec<ModuleKey>) -> Result<(), DepsError> {
    let imports = self.0.imports.iter().map(|il| &il.module);

    parents.push(key.clone());

    for module_path in imports {
      let module_key = ModuleKey(module_path.path.join("."));

      // check whether it’s already in the deps
      if deps.contains(&module_key) {
        continue;
      }

      // check whether the module was already visited
      if parents.contains(&module_key) {
        return Err(DepsError::Cycle(module_key.clone(), module_key.clone()));
      }

      // get the dependency module 
      let module = store.get(&module_key).ok_or_else(|| DepsError::LoadError(module_key.clone()))?;
      module.borrow().deps_no_cycle(store, &module_key, parents, deps)?;

      deps.push(module_key.clone());
      parents.pop();
    }

    Ok(())
  }

  /// Fold a module and its dependencies into a single module. The list of dependencies is also
  /// returned.
  pub fn gather(&self, store: &mut Store, key: &ModuleKey) -> Result<(Self, Vec<ModuleKey>), DepsError> {
    let deps = self.deps(store, key)?;
    let glsl =
      deps.iter()
          .flat_map(|kd| {
              let m = store.get(kd).unwrap();
              let g = m.borrow().0.glsl.clone();
              g
            })
          .chain(self.0.glsl.clone())
          .collect();

    let module = Module(SyntaxModule {
      imports: Vec::new(),
      glsl
    });

    Ok((module, deps))
  }

  /// Fold a module into its GLSL setup.
  pub fn to_glsl_setup(&self) -> Result<GLSLSetup, GLSLConversionError> {
    let uniforms = self.uniforms();
    let blocks = self.blocks();
    let structs = self.structs();
    let functions = self.functions();

    let mut common = String::new();
    let mut vs = String::new();
    let mut fs = String::new();

    // sink uniforms, blocks and structs first as a common framework
    for uniform in &uniforms {
      writer::glsl::show_single_declaration(&mut common, uniform);
      let _ = common.write_str(";\n");
    }

    for block in &blocks {
      writer::glsl::show_block(&mut common, block);
    }

    // filter out special functions so that we don’t put them in the common part
    for f in functions.iter().filter(|f| {
        let n: &str = &f.prototype.name;
        n != "map_vertex" && n != "map_frag_data"
      }) {
      writer::glsl::show_function_definition(&mut common, f)
    }

    // get the special functions
    let map_vertex = functions.iter().find(|fd| &fd.prototype.name == "map_vertex");
    let map_frag_data = functions.iter().find(|fd| &fd.prototype.name == "map_frag_data");

    match (map_vertex, map_frag_data) {
      (None, _) => return Err(GLSLConversionError::NoVertexShader),
      (_, None) => return Err(GLSLConversionError::NoFragmentShader),
      (Some(vf), Some(ff)) => {
        let (vertex_ret_ty, vertex_outputs) = sink_vertex_shader(&mut vs, vf, &structs)?;
        let fragment_ret_ty = sink_fragment_shader(&mut fs, ff, &structs, &vertex_ret_ty, &vertex_outputs)?;
        
        // stages don’t have the common structures yet because they might define overloaded ones, so
        let mut structs_str = String::new();
        for s in &structs {
          if s.name != vertex_ret_ty.name && s.name != fragment_ret_ty.name {
            writer::glsl::show_struct(&mut structs_str, s);
          }
        }

        common = structs_str + &common;
      }
    }

    if vs.is_empty() {
      Err(GLSLConversionError::NoVertexShader)
    } else if fs.is_empty() {
      Err(GLSLConversionError::NoFragmentShader)
    } else {
      let setup = GLSLSetup {
        vs: common.clone() + &vs,
        fs: common.clone() + &fs
      };

      Ok(setup)
    }
  }

  /// Get all the uniforms defined in a module.
  pub fn uniforms(&self) -> Vec<SingleDeclaration> {
    let mut uniforms = Vec::new();

    for glsl in &self.0.glsl {
      if let ExternalDeclaration::Declaration(Declaration::InitDeclaratorList(ref i)) = *glsl {
        if let Some(ref q) = i.head.ty.qualifier {
          if q.qualifiers.contains(&TypeQualifierSpec::Storage(StorageQualifier::Uniform)) {
            uniforms.push(i.head.clone());

            // check whether we have more
            for next in &i.tail {
              uniforms.push(SingleDeclaration {
                ty: i.head.ty.clone(),
                name: Some(next.name.clone()),
                array_specifier: next.array_specifier.clone(),
                initializer: None
              })
            }
          }
        }
      }
    }

    uniforms
  }

  /// Get all the blocks defined in a module.
  pub fn blocks(&self) -> Vec<Block> {
    self.0.glsl.iter().filter_map(|ed| {
      match *ed {
        ExternalDeclaration::Declaration(Declaration::Block(ref b)) => Some(b.clone()),
        _ => None
      }
    }).collect()
  }

  /// Get all the functions.
  pub fn functions(&self) -> Vec<FunctionDefinition> {
    self.0.glsl.iter().filter_map(|ed| match *ed {
      ExternalDeclaration::FunctionDefinition(ref def) => Some(def.clone()),
      _ => None
    }).collect()
  }

  /// Get all the declared structures.
  pub fn structs(&self) -> Vec<StructSpecifier> {
    self.0.glsl.iter().filter_map(|ed| {
      match *ed {
        ExternalDeclaration::Declaration(
          Declaration::InitDeclaratorList(
            InitDeclaratorList {
              head: SingleDeclaration {
                ty: FullySpecifiedType {
                  ty: TypeSpecifier {
                    ty: TypeSpecifierNonArray::Struct(ref s),
                    ..
                  },
                  ..
                },
                ..
              },
              ..
            }
          )
        ) => Some(s.clone()),
        _ => None
      }
    }).collect()
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GLSLConversionError {
  NoVertexShader,
  NoFragmentShader,
  OutputHasMainQualifier,
  ReturnTypeMustBeAStruct(TypeSpecifier),
  WrongOutputFirstField(StructFieldSpecifier),
  OutputFieldCannotBeStruct(usize, TypeSpecifier),
  OutputFieldCannotHaveSeveralIdentifiers(usize, StructFieldSpecifier),
  UnknownInputType(TypeName),
  NotSingleArgFn // FIXME: wat da fak?!
}

#[derive(Clone, Debug, PartialEq)]
pub struct GLSLSetup {
  pub vs: String,
  pub fs: String
}

/// Vertex shader I/O interface.
///
/// It contains the inputs and the outputs to the next stage.
#[derive(Clone, Debug, PartialEq)]
pub struct VertexShaderInterface {
  pub inputs: Vec<ExternalDeclaration>,
  pub outputs: Vec<ExternalDeclaration>
}

fn single_to_external_declaration(sd: SingleDeclaration) -> ExternalDeclaration {
  ExternalDeclaration::Declaration(
    Declaration::InitDeclaratorList(
      InitDeclaratorList {
        head: sd,
        tail: Vec::new()
      }
    )
  )
}

/// Sink a vertex shader.
fn sink_vertex_shader<F>(sink: &mut F,
                         map_vertex: &FunctionDefinition,
                         structs: &[StructSpecifier])
                         -> Result<(StructSpecifier, Vec<SingleDeclaration>), GLSLConversionError>
                         where F: Write {
  let inputs = vertex_shader_inputs(&map_vertex.prototype.parameters)?;
  let outputs = vertex_shader_outputs(&map_vertex.prototype.ty, structs)?;
  let ret_ty = get_fn_ret_ty(map_vertex, structs)?;

  for sd in inputs.iter().chain(&outputs) {
    let ed = single_to_external_declaration(sd.clone());
    writer::glsl::show_external_declaration(sink, &ed);
  }

  // sink the return type
  writer::glsl::show_struct(sink, &ret_ty);

  // sink the map_vertex function
  writer::glsl::show_function_definition(sink, map_vertex);

  // void main
  let _ = sink.write_str("void main() {\n  ");

  // call the map_vertex function
  let mut assigns = String::new();
  sink_vertex_shader_output(sink, &mut assigns, &ret_ty);

  let _ = sink.write_str(" v = map_vertex(");
  sink_vertex_shader_input_args(sink, &map_vertex);
  let _ = sink.write_str(");\n");

  // assign to outputs
  let _ = sink.write_str(&assigns);

  // end of the main function
  let _ = sink.write_str("}\n\n");

  Ok((ret_ty, outputs))
}

fn get_fn_ret_ty(f: &FunctionDefinition, structs: &[StructSpecifier]) -> Result<StructSpecifier, GLSLConversionError> {
  if let TypeSpecifierNonArray::TypeName(ref name) = f.prototype.ty.ty.ty {
    if let Some(ref ty) = structs.iter().find(|ref s| s.name.as_ref() == Some(name)) {
      Ok((*ty).clone())
    } else {
      Err(GLSLConversionError::ReturnTypeMustBeAStruct(f.prototype.ty.ty.clone()))
    }
  } else {
    Err(GLSLConversionError::ReturnTypeMustBeAStruct(f.prototype.ty.ty.clone()))
  }
}

fn get_fn_input_ty_name(f: &FunctionDefinition) -> Result<TypeName, GLSLConversionError> {
  match f.prototype.parameters.as_slice() {
    &[FunctionParameterDeclaration::Named(_, FunctionParameterDeclarator {
      ty: TypeSpecifier {
        ty: TypeSpecifierNonArray::TypeName(ref n),
        ..
      },
      ..
    })] => Ok(n.clone()),

    &[FunctionParameterDeclaration::Unnamed(_, TypeSpecifier {
      ty: TypeSpecifierNonArray::TypeName(ref n),
      ..
    })] => Ok(n.clone()),

    _ => Err(GLSLConversionError::NotSingleArgFn)
  }
}

/// Sink a vertex shader’s output.
fn sink_vertex_shader_output<F, G>(sink: &mut F, assigns: &mut G, ty: &StructSpecifier) where F: Write, G: Write {
  if let Some(ref name) = ty.name {
    let _ = sink.write_str(name);
  } else {
    panic!("cannot happen");
  }

  let _ = assigns.write_str("  gl_Position = v.gl_Position;\n");

  for field in &ty.fields[1..] {
    for &(ref identifier, _) in &field.identifiers {
      let _ = write!(assigns, "  __v_{0} = v.{0};\n", identifier);
    }
  }
}

/// Sink the arguments of the map_vertex function.
fn sink_vertex_shader_input_args<F>(sink: &mut F, map_vertex: &FunctionDefinition) where F: Write {
  let args = &map_vertex.prototype.parameters;

  if !args.is_empty() {
    // sink the first argument upfront
    let first_arg = &args[0];

    sink_vertex_shader_input_arg(sink, first_arg);

    for arg in &map_vertex.prototype.parameters[1..] {
      let _ = sink.write_str(", ");
      sink_vertex_shader_input_arg(sink, arg);
    }
  }
}

/// Sink an argument of a function.
fn sink_vertex_shader_input_arg<F>(sink: &mut F, arg: &FunctionParameterDeclaration) where F: Write {
  match *arg {
    FunctionParameterDeclaration::Named(_, ref d) => {
      let _ = sink.write_str(&d.name);
    }
    _ => ()
  }
}


fn vertex_shader_inputs<'a, I>(args: I) -> Result<Vec<SingleDeclaration>, GLSLConversionError> where I: IntoIterator<Item = &'a FunctionParameterDeclaration> {
  let mut inputs = Vec::new();

  for (i, arg) in args.into_iter().enumerate() {
    match *arg {
      FunctionParameterDeclaration::Named(ref ty_qual, ref decl) => {
        let layout_qualifier = LayoutQualifier {
          ids: vec![LayoutQualifierSpec::Identifier("location".to_owned(), Some(Box::new(Expr::IntConst(i as i32))))]
        };
        let base_qualifier = TypeQualifier {
          qualifiers: vec![
            TypeQualifierSpec::Layout(layout_qualifier),
            TypeQualifierSpec::Storage(StorageQualifier::In)
          ]
        };
        let qualifier = match *ty_qual {
          Some(ref qual) => TypeQualifier { qualifiers: base_qualifier.qualifiers.into_iter().chain(qual.clone().qualifiers).collect() },
          None => base_qualifier
        };
        let ty = decl.ty.clone();
        let name = Some(decl.name.clone());
        let array_spec = decl.array_spec.clone();
        let sd = 
          SingleDeclaration {
            ty: FullySpecifiedType {
              qualifier: Some(qualifier),
              ty
            },
            name,
            array_specifier: array_spec,
            initializer: None
          };

        inputs.push(sd);
      }

      // unnamed arguments is not an error! it serves when the argument is not used, but we still
      // need to state how the data is stored in the buffer
      _ => ()
    }
  }

  Ok(inputs)
}

fn vertex_shader_outputs(fsty: &FullySpecifiedType, structs: &[StructSpecifier]) -> Result<Vec<SingleDeclaration>, GLSLConversionError> {
  // we refuse that the output has a main qualifier
  if fsty.qualifier.is_some() {
    return Err(GLSLConversionError::OutputHasMainQualifier);
  }

  let ty = &fsty.ty;

  // we enforce that the output must be a struct that follows a certain pattern
  match ty.ty {
    TypeSpecifierNonArray::TypeName(ref ty_name) => {
      let real_ty = structs.iter().find(|ref s| s.name.as_ref() == Some(ty_name));

      match real_ty {
        Some(ref s) => {
          // the first field must be named "gl_Position", has type vec4 and no qualifier
          let first_field = &s.fields[0];

          if first_field.qualifier.is_some() ||
             first_field.ty.ty != TypeSpecifierNonArray::Vec4 ||
             first_field.identifiers != vec![("gl_Position".to_owned(), None)] {
            return Err(GLSLConversionError::WrongOutputFirstField(first_field.clone()));
          }

          // then, for all other fields, we check that they are not composite type (i.e. structs); if
          // they are not, add them to the interface; otherwise, fail
          fields_to_single_decls(&s.fields[1..], "__v_")
        }
        _ => Err(GLSLConversionError::ReturnTypeMustBeAStruct(ty.clone()))
      }
    }
    _ => Err(GLSLConversionError::ReturnTypeMustBeAStruct(ty.clone()))
  }
}

/// Map a struct’s fields to a Vec<ExternalDeclaration>. Typically suitable for generating outputs
/// from a struct fields.
fn fields_to_single_decls(fields: &[StructFieldSpecifier], prefix: &str) -> Result<Vec<SingleDeclaration>, GLSLConversionError> {
  let mut outputs = Vec::new();

  for (i, field) in fields.into_iter().enumerate() {
    if let TypeSpecifierNonArray::Struct(_) = field.ty.ty {
      return Err(GLSLConversionError::OutputFieldCannotBeStruct(i, field.ty.clone()));
    }

    // TODO: also fail on TypeSpecifierNonArray::TypeName

    if field.identifiers.len() > 1 {
      return Err(GLSLConversionError::OutputFieldCannotHaveSeveralIdentifiers(i, field.clone()));
    }

    outputs.push(field_to_single_decl(&field, prefix));
  }

  Ok(outputs)
}

/// Map a StructFieldSpecifier to an ExternalDeclaration. Typically suitable for generating an
/// output from a struct field.
fn field_to_single_decl(field: &StructFieldSpecifier, prefix: &str) -> SingleDeclaration {
  let base_qualifier = TypeQualifierSpec::Storage(StorageQualifier::Out);
  let qualifier = match field.qualifier {
    Some(ref qual) => TypeQualifier { qualifiers: qual.clone().qualifiers.into_iter().chain(once(base_qualifier)).collect() },
    None => TypeQualifier { qualifiers: vec![base_qualifier] }
  };
  let fsty = FullySpecifiedType {
    qualifier: Some(qualifier),
    ty: field.ty.clone()
  };

  SingleDeclaration {
    ty: fsty,
    name: Some(prefix.to_owned() + &field.identifiers[0].0),
    array_specifier: field.identifiers[0].1.clone(),
    initializer: None
  }
}

/// Sink a fragment shader.
fn sink_fragment_shader<F>(sink: &mut F,
                           map_frag_data: &FunctionDefinition,
                           structs: &[StructSpecifier],
                           prev_ret_ty: &StructSpecifier,
                           prev_inputs: &[SingleDeclaration])
                           -> Result<StructSpecifier, GLSLConversionError>
                           where F: Write {
  let input_ty_name = get_fn_input_ty_name(map_frag_data)?;

  // ensure we use the right input type
  if Some(&input_ty_name) != prev_ret_ty.name.as_ref() {
    return Err(GLSLConversionError::UnknownInputType(input_ty_name.clone()));
  }

  let inputs = fragment_shader_inputs(prev_inputs); // this is wrong, need to adapt the previous inputs instead
  let ret_ty = get_fn_ret_ty(map_frag_data, structs)?;
  let outputs = fields_to_single_decls(&ret_ty.fields, "__f_")?;

  // sink inputs and outputs
  for sd in inputs.iter().chain(&outputs) {
    let ed = single_to_external_declaration(sd.clone());
    writer::glsl::show_external_declaration(sink, &ed);
  }

  // sink the previous return type (from the previous stage)
  writer::glsl::show_struct(sink, prev_ret_ty);

  // sink the return type
  writer::glsl::show_struct(sink, &ret_ty);

  // sink the map_frag_data function
  writer::glsl::show_function_definition(sink, map_frag_data);

  // void main
  let _ = sink.write_str("void main() {\n  ");

  let _ = write!(sink, "{0} i = {0}(", prev_ret_ty.name.as_ref().unwrap());

  let _ = sink.write_str(inputs[0].name.as_ref().unwrap());

  for input in &inputs[1..] {
    let _ = write!(sink, ", {}", input.name.as_ref().unwrap());
  }

  let _ = sink.write_str(");\n");
  let _ = write!(sink, "  {} o = map_frag_data(i);\n", ret_ty.name.as_ref().unwrap());

  for (output, ret_ty_field) in outputs.iter().zip(&ret_ty.fields) {
    let _ = write!(sink, "  {} = o.{};\n", output.name.as_ref().unwrap(), ret_ty_field.identifiers[0].0);
  }

  // end of the main function
  let _ = sink.write_str("}\n\n");

  Ok(ret_ty)
}

/// Replace an input declaration by its output declaration dual.
fn replace_out_in_single_declaration(input: SingleDeclaration) -> SingleDeclaration {
  let qualifier = input.ty.qualifier.map(|q| {
    TypeQualifier {
      qualifiers: q.qualifiers.into_iter().map(|qs| {
        match qs {
          TypeQualifierSpec::Storage(StorageQualifier::Out) => TypeQualifierSpec::Storage(StorageQualifier::In),
          _ => qs
        }
      }).collect()
    }
  });

  SingleDeclaration {
    ty: FullySpecifiedType {
      qualifier,
      .. input.ty
    },
    .. input
  }
}

fn fragment_shader_inputs(inputs: &[SingleDeclaration]) -> Vec<SingleDeclaration> {
  inputs.into_iter().map(|sd| replace_out_in_single_declaration(sd.clone())).collect()
}

/// Class of errors that can happen in dependencies.
#[derive(Clone, Debug, PartialEq)]
pub enum DepsError {
  /// If a module’s dependencies has any cycle, the dependencies are unusable and the cycle is
  /// returned.
  Cycle(ModuleKey, ModuleKey),
  /// There was a loading error of a module.
  LoadError(ModuleKey)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleKey(String);

impl ModuleKey {
  pub fn new(key: &str) -> Self {
    ModuleKey(key.to_owned())
  }
}

impl CacheKey for ModuleKey {
  type Target = Module;
}

impl StoreKey for ModuleKey {
  fn key_to_path(&self) -> PathBuf {
    PathBuf::from(self.0.replace(".", "/") + ".spsl")
  }
}

impl Load for Module {
  fn load<K>(key: &K, _: &mut Store) -> Result<LoadResult<Self>, LoadError> where K: StoreKey<Target = Self> {
    let path = key.key_to_path();

    let mut fh = File::open(&path).map_err(|_| LoadError::FileNotFound(path.into()))?;
    let mut src = String::new();
    let _ = fh.read_to_string(&mut src);

    match parser::parse_str(&src[..], parser::module) {
      parser::ParseResult::Ok(module) => {
        Ok(Module(module).into())
      }
      parser::ParseResult::Err(e) => Err(LoadError::ConversionFailed(format!("{:?}", e))),
      _ => Err(LoadError::ConversionFailed("incomplete input".to_owned()))
    }
  }
}

