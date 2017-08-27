//! Shader module.
//!
//! Shader functions and declarations can be grouped in so-called *modules*. Modules structure is
//! inherently tied to the filesystem’s tree.
//!
//! You’re not supposed to use modules at the Rust level, even though you can. You’re supposed to
//! actually write modules that will be used by shader programs.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use render::shader::lang::parser;
use render::shader::lang::syntax::{Declaration, ExternalDeclaration, FunctionDefinition, FullySpecifiedType,
                                   FunctionParameterDeclaration, InitDeclaratorList,
                                   Module as SyntaxModule, SingleDeclaration, StorageQualifier,
                                   TypeQualifierSpec};
use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};

/// Shader module.
///
/// A shader module is a piece of GLSL code with optional import lists (dependencies).
///
/// You’re not supposed to directly manipulate any object of this type. You just write modules on
/// disk and let everyting happen automatically for you.
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
  pub fn gather(&self, store: &mut Store, k: &ModuleKey) -> Result<(Self, Vec<ModuleKey>), DepsError> {
    let deps = self.deps(store, k)?;
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

  /// Get all the functions.
  pub fn functions(&self) -> Vec<FunctionDefinition> {
    self.0.glsl.iter().filter_map(|ed| match *ed {
      ExternalDeclaration::FunctionDefinition(ref def) => Some(def.clone()),
      _ => None
    }).collect()
  }
}

/// Vertex shader I/O interface.
///
/// It contains the inputs and the outputs to the next stage.
#[derive(Clone, Debug, PartialEq)]
pub struct VertexShaderInterface {
  inputs: Vec<ExternalDeclaration>,
  outputs: FullySpecifiedType
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub enum VertexShaderInterfaceError {
  UnnamedInput
}

/// Build the vertex shader interface from a function definition.
pub fn vertex_shader_interface(fun_def: &FunctionDefinition) -> Result<VertexShaderInterface, VertexShaderInterfaceError> {
  let proto = &fun_def.prototype;
  let inputs = vertex_shader_interface_inputs(proto.parameters.iter())?;
  let outputs = proto.ty.clone();

  Ok(VertexShaderInterface { inputs, outputs })
}

fn vertex_shader_interface_inputs<'a, I>(args: I) -> Result<Vec<ExternalDeclaration>, VertexShaderInterfaceError> where I: IntoIterator<Item = &'a FunctionParameterDeclaration> {
  let mut inputs = Vec::new();

  for arg in args {
    match *arg {
      FunctionParameterDeclaration::Unnamed(..) => return Err(VertexShaderInterfaceError::UnnamedInput),
      FunctionParameterDeclaration::Named(ref ty_qual, ref decl) => {
        let qualifier = ty_qual.clone();
        let ty = decl.ty.clone();
        let name = Some(decl.name.clone());
        let array_spec = decl.array_spec.clone();
        let idl = InitDeclaratorList {
          head: SingleDeclaration {
            ty: FullySpecifiedType {
              qualifier,
              ty
            },
            name,
            array_specifier: array_spec,
            initializer: None
          },
          tail: Vec::new()
        };
        let ed = ExternalDeclaration::Declaration(Declaration::InitDeclaratorList(idl));

        inputs.push(ed);
      }
    }
  }

  Ok(inputs)
}

fn vertex_shader_interface_outputs(ty: &FullySpecifiedType) -> Result<Vec<ExternalDeclaration>, VertexShaderInterface> {
  let mut outputs = Vec::new();
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

