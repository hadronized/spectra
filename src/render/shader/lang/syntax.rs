//! Syntax of the language.
//!
//! For now, most of the language is an EDSL describing an augmented GLSL with a few keywords.
pub use glsl::syntax::*;

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

type ModuleName = String;
