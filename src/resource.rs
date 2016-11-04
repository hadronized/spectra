use std::path::{Path, PathBuf};

/// Class of types that can be loaded.
pub trait Load: Sized {
  type Args;

  // TODO: see whether we can use something with From/Into instead, so that we can use lambdas.
  fn load<P>(path: P, args: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path>;
}

/// Class of types that can be reloaded.
pub trait Reload: Load {
  fn reload<P>(&self, path :P) -> Result<Self, LoadError> where P: AsRef<Path>;
}

/// Default implementation for types which are loaded without any arguments.
impl<T> Reload for T where T: Load<Args=()> {
  fn reload<P>(&self, path :P) -> Result<Self, LoadError> where P: AsRef<Path> {
    Self::load(path, ())
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
  FileNotFound(PathBuf, String),
  ParseFailed(String),
  ConversionFailed(String)
}
