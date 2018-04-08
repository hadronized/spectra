//! Syntax of the language.
//!
//! For now, most of the language is an EDSL describing an augmented GLSL with a few keywords.
//!
//! This module also exports some useful functions to cope with the syntax types.

pub use glsl::syntax::*;
use glsl::writer;
use std::error::Error;
use std::fmt::{self, Write};
use std::iter::once;
use std::path::{Path, PathBuf};

pub type GLSL = Vec<ExternalDeclaration>;

/// A module.
///
/// A module has a list of imports and a list of GLSL extern declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
  /// List of imports for this module.
  pub imports: Vec<ImportList>,
  /// The GLSL body of the module.
  pub glsl: GLSL
}

/// A non-empty import list.
///
/// It consists of a module path, like `Foo.Bar.Zoo`, and a list of symbols to load from that path,
/// as in `rick, marty, doggo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportList {
  /// The module path to load symbols from.
  pub module: ModulePath,
  /// List of symbols to import.
  pub list: Vec<ModuleSymbol>
}

impl ImportList {
  /// Generate a [PathBuf] that represents this import on disk.
  pub fn to_path(&self, root: &Path) -> PathBuf {
    PathBuf::from(root.join(self.module.path.join("/") + ".chdr"))
  }
}

/// A module path is a non-empty list of module(s), representing a hierarchy.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ModulePath {
  /// Hierarchical path of modules leading to the module we want to import (included in the path).
  pub path: Vec<ModuleName>
}

/// Name of a module.
pub type ModuleName = String;

/// A symbol, living in a module.
pub type ModuleSymbol = String;

pub type ExpectedNumberOfArgs = usize;
pub type FoundNumberOfArgs = usize;

/// GLSL conversion error.
///
/// Such an errors can happen when a module is ill-formed.
#[derive(Clone, Debug, PartialEq)]
pub enum GLSLConversionError {
  /// No vertex shader was found. A vertex shader is required in order to build a shading pipeline.
  NoVertexShader,
  /// No fragment shader was found. A fragment shader is required in order to build a shading pipeline.
  NoFragmentShader,
  /// The output must not have a qualifier.
  OutputHasMainQualifier,
  /// The returned value must not be a struct.
  ReturnTypeMustBeAStruct(TypeSpecifier),
  /// The first field has the wrong type.
  WrongOutputFirstField(StructFieldSpecifier),
  /// The field of a type used as output cannot be a struct.
  ///
  /// This variant also gives the index of the field.
  OutputFieldCannotBeStruct(usize, StructSpecifier),
  /// The field of a type used as output cannot be a type name.
  ///
  /// This variant also gives the index of the field.
  OutputFieldCannotBeTypeName(usize, TypeName),
  /// The field of a type used as output cannot have several identifiers (only one).
  ///
  /// This variant also gives the index of the field.
  OutputFieldCannotHaveSeveralIdentifiers(usize, StructFieldSpecifier),
  /// The input type is unknown.
  UnknownInputType(TypeName),
  /// Wrong number of arguments.
  WrongNumberOfArgs(ExpectedNumberOfArgs, FoundNumberOfArgs),
  /// The type is not a required type name.
  NotTypeName,
  /// The geometry input is wrong.
  WrongGeometryInput,
  /// The geometry input’s dimension is wrong.
  WrongGeometryInputDim(usize),
  /// The geometry output layout is wrong.
  WrongGeometryOutputLayout(Option<TypeQualifier>)
}

impl fmt::Display for GLSLConversionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}


impl Error for GLSLConversionError {
  fn description(&self) -> &str {
    match *self {
      GLSLConversionError::NoVertexShader => "no vertex shader",
      GLSLConversionError::NoFragmentShader => "no fragment shader",
      GLSLConversionError::OutputHasMainQualifier => "output has main qualifier(s)",
      GLSLConversionError::ReturnTypeMustBeAStruct(_) => "return type is not a struct",
      GLSLConversionError::WrongOutputFirstField(_) => "first field’s type is forbidden as output",
      GLSLConversionError::OutputFieldCannotBeStruct(..) => "output field cannot be a struct",
      GLSLConversionError::OutputFieldCannotBeTypeName(..) => "output field cannot be a type name",
      GLSLConversionError::OutputFieldCannotHaveSeveralIdentifiers(..) => "output field cannot have several identifiers",
      GLSLConversionError::UnknownInputType(_) => "unknown input type",
      GLSLConversionError::WrongNumberOfArgs(..) => "wrong number of arguments",
      GLSLConversionError::NotTypeName => "not a type name",
      GLSLConversionError::WrongGeometryInput => "wrong geometry input",
      GLSLConversionError::WrongGeometryInputDim(_) => "wrong geometry input’s dimension",
      GLSLConversionError::WrongGeometryOutputLayout(_) => "wrong geometry output layout"
    }
  }
}

/// Sink single declarations as external declarations.
pub fn sink_single_as_ext_decls<'a, F, I>(sink: &mut F, s: I)
                                      where I: IntoIterator<Item = &'a SingleDeclaration>,
                                            F: Write {
  for sd in s {
    let ed = single_to_external_declaration(sd.clone());
    writer::glsl::show_external_declaration(sink, &ed);
  }
}

/// Turn a `SingleDeclaration` into an `ExternalDeclaration`.
pub fn single_to_external_declaration(sd: SingleDeclaration) -> ExternalDeclaration {
  ExternalDeclaration::Declaration(
    Declaration::InitDeclaratorList(
      InitDeclaratorList {
        head: sd,
        tail: Vec::new()
      }
    )
  )
}

/// Replace an output declaration by its input declaration dual.
///
/// Useful when an input interface must match an output one.
pub fn input_from_output(output: SingleDeclaration, has_array: bool) -> SingleDeclaration {
  let
    qualifier = output.ty.qualifier.map(|q| {
      TypeQualifier {
        qualifiers: q.qualifiers.into_iter().map(|qs| {
          match qs {
            TypeQualifierSpec::Storage(StorageQualifier::Out) =>
              TypeQualifierSpec::Storage(StorageQualifier::In),
            _ => qs
          }
        }).collect()
      }
    });
  let ty =
    TypeSpecifier {
      array_specifier: if has_array { Some(ArraySpecifier::Unsized) } else { None },
      .. output.ty.ty
    };

  SingleDeclaration {
    ty: FullySpecifiedType { qualifier, ty },
    .. output 
  }
}

/// Replace outputs by inputs.
pub fn inputs_from_outputs(outputs: &[SingleDeclaration], has_array: bool) -> Vec<SingleDeclaration> {
  outputs.into_iter().map(|sd| input_from_output(sd.clone(), has_array)).collect()
}

/// Map a StructFieldSpecifier to an ExternalDeclaration.
///
/// Typically suitable for generating an output from a struct field.
pub fn field_to_single_decl(field: &StructFieldSpecifier, prefix: &str) -> SingleDeclaration {
  let base_qualifier = TypeQualifierSpec::Storage(StorageQualifier::Out);
  let qualifier = match field.qualifier {
    Some(ref qual) =>
      TypeQualifier {
        qualifiers: qual.clone().qualifiers.into_iter().chain(once(base_qualifier)).collect()
      },
    None => TypeQualifier {
      qualifiers: vec![base_qualifier]
    }
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

/// Map a struct’s fields to a Vec<ExternalDeclaration>.
///
/// Typically suitable for generating outputs from a struct fields.
pub fn fields_to_single_decls(fields: &[StructFieldSpecifier], prefix: &str)
                              -> Result<Vec<SingleDeclaration>, GLSLConversionError> {
  let mut outputs = Vec::new();

  for (i, field) in fields.into_iter().enumerate() {
    match field.ty.ty {
      TypeSpecifierNonArray::Struct(ref s) => {
        return Err(GLSLConversionError::OutputFieldCannotBeStruct(i + 1, s.clone()));
      }
      TypeSpecifierNonArray::TypeName(ref t) => {
        return Err(GLSLConversionError::OutputFieldCannotBeTypeName(i + 1, t.clone()));
      }
      _ => ()
    }

    if field.identifiers.len() > 1 {
      return Err(GLSLConversionError::OutputFieldCannotHaveSeveralIdentifiers(i + 1, field.clone()));
    }

    outputs.push(field_to_single_decl(&field, prefix));
  }

  Ok(outputs)
}

/// Filter out a function definition by removing its unused arguments.
pub fn remove_unused_args_fn(f: &FunctionDefinition) -> FunctionDefinition {
  let f = f.clone();

  FunctionDefinition {
    prototype: FunctionPrototype {
      parameters: f.prototype.parameters.into_iter().filter(is_fn_arg_named).collect(),
      .. f.prototype
    },
    .. f
  }
}

pub fn is_fn_arg_named(arg: &FunctionParameterDeclaration) -> bool {
  if let FunctionParameterDeclaration::Named(..) = *arg {
    true
  } else {
    false
  }
}

/// Extract the type name of a function argument. If the argument’s type is not a typename,
/// nothing is returned.
/// Get the fully specified type of a function’s argument.
pub fn fn_arg_as_fully_spec_ty(arg: &FunctionParameterDeclaration) -> FullySpecifiedType {
  match *arg {
    FunctionParameterDeclaration::Named(ref qualifier, FunctionParameterDeclarator {
      ref ty,
      ..
    }) => FullySpecifiedType {
      qualifier: qualifier.clone(),
      ty: ty.clone()
    },
    FunctionParameterDeclaration::Unnamed(ref qualifier, ref ty) => {
      FullySpecifiedType {
        qualifier: qualifier.clone(),
        ty: ty.clone()
      }
    }
  }
}

/// Extract the type name of a fully specified type. If the type is not a typename, nothing is
/// returned.
pub fn get_ty_name_from_fully_spec_ty(fst: &FullySpecifiedType) -> Result<TypeName, GLSLConversionError> {
  if let TypeSpecifierNonArray::TypeName(ref n) = fst.ty.ty {
    Ok(n.clone())
  } else {
    Err(GLSLConversionError::NotTypeName)
  }
}

/// Get the type name of the argument of a unary function. If the argument is not unary, fail
/// with the approriate error.
pub fn get_fn1_input_ty_name(f: &FunctionDefinition) -> Result<TypeName, GLSLConversionError> {
  let slice = f.prototype.parameters.as_slice();
  match slice {
    &[ref arg] => {
      let fst = fn_arg_as_fully_spec_ty(arg);
      get_ty_name_from_fully_spec_ty(&fst)
    }
    _ => Err(GLSLConversionError::WrongNumberOfArgs(1, slice.len()))
  }
}

/// Get the return type of a function by looking up its definition in the provided slice.
pub fn get_fn_ret_ty(f: &FunctionDefinition, structs: &[StructSpecifier]) -> Result<StructSpecifier, GLSLConversionError> {
  struct_from_ty_spec(&f.prototype.ty.ty, structs)
}

/// Get the struct definition associated with a type specifier.
pub fn struct_from_ty_spec(
  ty_spec: &TypeSpecifier,
  structs: &[StructSpecifier]
) -> Result<StructSpecifier, GLSLConversionError> {
  if let TypeSpecifierNonArray::TypeName(ref name) = ty_spec.ty {
    if let Some(ref ty) = structs.iter().find(|ref s| s.name.as_ref() == Some(name)) {
      Ok((*ty).clone())
    } else {
      Err(GLSLConversionError::ReturnTypeMustBeAStruct(ty_spec.clone()))
    }
  } else {
    Err(GLSLConversionError::ReturnTypeMustBeAStruct(ty_spec.clone()))
  }
}

/// Drop the first field of a struct.
pub fn drop_1st_field(s: &StructSpecifier) -> StructSpecifier {
  StructSpecifier {
    name: s.name.clone(),
    fields: s.fields.iter().skip(1).cloned().collect(),
  }
}
