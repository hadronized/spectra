//! Error that might occur while loading a resource.

use std::fmt;
use std::path::PathBuf;

/// Error that might occur while loading resources.
#[derive(Clone, Debug)]
pub enum Error {
  /// The resource couldn’t be loaded from the file system.
  CannotLoadFromFS(PathBuf, Reason),
  /// The resource couldn’t be loaded.
  CannotLoadFromLogical(String, Reason),
}

/// Reason of a load failure.
pub type Reason = String;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Error::CannotLoadFromFS(ref path, ref reason) =>
        write!(f, "cannot load {} from file system: {}", path.display(), reason),

      Error::CannotLoadFromLogical(ref s, ref reason) =>
        write!(f, "cannot load <{}>: {}", s, reason)
    }
  }
}
