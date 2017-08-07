//! Syntax of the language.
//!
//! For now, most of the language is an EDSL describing an augmented GLSL with a few keywords.
use glsl::syntax::ExternalDeclaration;

/// Our shading language.
#[derive(Clone, Debug, PartialEq)]
enum Lang {
  /// An `export list_of_identifiers_` statement.
  Export(ExportList),
  /// A `from module import list of identifiers` statement.
  Import(ImportList),
  /// A GLSL external declaration.
  GLSL(ExternalDeclaration)
}

pub type Identifier = String;

/// An non-empty export list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportList {
  pub export_list: Vec<ModulePath>
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

type ModuleName = String;
