//! Syntax of the language.
//!
//! For now, most of the language is an EDSL describing an augmented GLSL with a few keywords.
//!
//! This module also exports some useful functions to cope with the syntax types.

pub use glsl::syntax::*;
use glsl::writer;
use std::fmt::Write;
use std::iter::once;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
  pub imports: Vec<ImportList>,
  pub glsl: Vec<ExternalDeclaration>
}

/// A non-empty import list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportList {
  pub module: ModulePath,
  pub list: Vec<ModulePath>
}

/// A module path is a list of module(s), representing a hierarchy.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ModulePath {
  pub path: Vec<ModuleName>
}

pub type ModuleName = String;

pub type ExpectedNumberOfArgs = usize;
pub type FoundNumberOfArgs = usize;

/// GLSL conversion error.
///
/// Such an errors can happen when a module is ill-formed.
#[derive(Clone, Debug, PartialEq)]
pub enum GLSLConversionError {
  NoVertexShader,
  NoFragmentShader,
  OutputHasMainQualifier,
  ReturnTypeMustBeAStruct(TypeSpecifier),
  WrongOutputFirstField(StructFieldSpecifier),
  OutputFieldCannotBeStruct(usize, StructSpecifier),
  OutputFieldCannotBeTypeName(usize, TypeName),
  OutputFieldCannotHaveSeveralIdentifiers(usize, StructFieldSpecifier),
  UnknownInputType(TypeName),
  WrongNumberOfArgs(ExpectedNumberOfArgs, FoundNumberOfArgs),
  NotTypeName,
  WrongGeometryOutputLayout
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

/// Drop the first field of a struct.
pub fn drop_first_field(s: &StructSpecifier) -> StructSpecifier {
  StructSpecifier {
    name: s.name.clone(),
    fields: s.fields.iter().skip(1).cloned().collect(),
  }
}

/// Replace an output declaration by its input declaration dual.
///
/// Useful when an input interface must match an output one.
pub fn input_from_output(output: SingleDeclaration) -> SingleDeclaration {
  let qualifier = output.ty.qualifier.map(|q| {
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

  SingleDeclaration {
    ty: FullySpecifiedType {
      qualifier,
      .. output.ty
    },
    .. output 
  }
}

/// Replace outputs by inputs.
pub fn inputs_from_outputs(outputs: &[SingleDeclaration]) -> Vec<SingleDeclaration> {
  outputs.into_iter().map(|sd| input_from_output(sd.clone())).collect()
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
pub fn get_ty_name_from_full_spec_ty(fst: &FullySpecifiedType) -> Option<TypeName> {
  if let TypeSpecifierNonArray::TypeName(ref n) = fst.ty.ty {
    Some(n.clone())
  } else {
    None
  }
}

/// Get the type name of the argument of a unary function. If the argument is not unary, fail
/// with the approriate error.
pub fn get_fn1_input_ty_name(f: &FunctionDefinition) -> Result<TypeName, GLSLConversionError> {
  let slice = f.prototype.parameters.as_slice();
  match slice {
    &[ref arg] => {
      let fst = fn_arg_as_fully_spec_ty(arg);
      get_ty_name_from_full_spec_ty(&fst).ok_or(GLSLConversionError::NotTypeName)
    }
    _ => Err(GLSLConversionError::WrongNumberOfArgs(1, slice.len()))
  }
}

/// Get the return type of a function by looking up its definition in the provided slice.
pub fn get_fn_ret_ty(f: &FunctionDefinition, structs: &[StructSpecifier]) -> Result<StructSpecifier, GLSLConversionError> {
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
