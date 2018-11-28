//! Demo runner.

use std::fmt;

pub mod debug;

/// Possible runner errors.
#[derive(Debug)]
pub enum Error {
  CannotCreateWindow(String),
  CannotCreateStore(String),
  DemoInitializationFailure(String)
}

impl Error {
  pub(crate) fn cannot_create_window<R>(reason: R) -> Self where R: Into<String> {
    Error::CannotCreateWindow(reason.into())
  }

  pub(crate) fn cannot_create_store<R>(reason: R) -> Self where R: Into<String> {
    Error::CannotCreateStore(reason.into())
  }

  pub(crate) fn demo_initialization_failure<R>(reason: R) -> Self where R: Into<String> {
    Error::DemoInitializationFailure(reason.into())
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Error::CannotCreateWindow(ref reason) => write!(f, "cannot create window: {}", reason),
      Error::CannotCreateStore(ref reason) => write!(f, "cannot create store: {}", reason),
      Error::DemoInitializationFailure(ref reason) => write!(f, "demo failed to initialize: {}", reason),
    }
  }
}
