//! Syntax of the language.
//!
//! For now, most of the language is an EDSL describing an augmented GLSL with a few keywords.
//! The idea is to converge to something able to recognize all tokens used in the source input but
//! in order to make things easier at first, we’ll consider some “opaque” code that we won’t be
//! using as tokens. For instance, if the user defines a function that is not a special function
//! we’re interested in, we’ll just parse the whole of it until we hit the end of its body.
use std::collections::HashMap;

/// A shader module.
///
/// A shader module is a container that associates some shading code to several identifiers.
struct ShaderModule {
  symbols: HashMap<Identifier, String>
}

/// Token.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
  /// An `export list_of_identifiers_` statement.
  Export(ExportList),
  /// A `from module import list of identifiers` statement.
  Import(ImportList),
  /// A `pipeline { list_of_pipeline_attributes }` statement.
  Pipeline(PipelineStatement),
  /// A yield statement, valid in geometry shaders.
  Yield(GeometryYieldExpression),
  /// A function definition.
  FunDef(FunSig, FunBody)
}

pub type Identifier = String;
type Expression = String;

/// An export non-empty list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportList {
  pub export_list: Vec<ModulePath>
}

/// An import non-empty list.
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

type ModuleName = String;

/// A pipeline statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineStatement {
  pub attributes: Vec<PipelineAttribute>
}

/// Attributes that can be set in a pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineAttribute {
  /// Maximum vertices that the geometry shader can output.
  GeometryShaderMaxVertices(u32),
  /// Number of times the geometry shader must be invoked.
  GeometryShaderInvokations(u32)
}

/// Expressions that can be yielded in a geometry shader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeometryYieldExpression {
  /// Yield a primitive.
  YieldPrimitive,
  /// Yield a primitive’s vertex (fold vertex).
  YieldFoldVertex(Expression)
}

/// A function signature.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FunSig {
  ret_ty: Option<RetTy>,
  ident: Identifier,
  args: FunArgList
}

// TODO: use an enum here instead
pub type RetTy = String;

/// Arguments list of a function signature.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FunArgList {
  list: Vec<FunArg>
}

/// An argument in the argument list of a function
#[derive(Clone, Debug, Eq, PartialEq)]
struct FunArg {
  qualifier: Option<FunArgQualifier>,
  ty: FunArgTy,
  ident: Identifier
}

// TODO: use enums here instead
type FunArgQualifier = String;
type FunArgTy = String;

type FunBody = String;
