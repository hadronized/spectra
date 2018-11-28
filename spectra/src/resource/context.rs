//! Resource context.

use crate::logger::{Logger, StdoutLogger};

/// Class of accepted contexts.
pub trait Context {
  type Logger: Logger;

  /// Get access to the internal logger.
  fn logger(&mut self) -> &mut Self::Logger;
}

#[derive(Debug)]
pub struct DefaultContext {
  logger: StdoutLogger
}

impl DefaultContext {
  pub fn new() -> Self {
    DefaultContext {
      logger: StdoutLogger
    }
  }
}

impl Default for DefaultContext {
  fn default() -> Self {
    DefaultContext::new()
  }
}

/// Using the default context will use:
///
///   - The `StdoutLogger` as a logger.
impl Context for DefaultContext {
  type Logger = StdoutLogger;

  fn logger(&mut self) -> &mut Self::Logger {
    &mut self.logger
  }
}
