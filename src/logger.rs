//! Logger.

use chrono::{Datelike, Local, Timelike};
use std::fmt::Arguments;

/// Trait use to log activity.
pub trait Logger {
  /// Log some information.
  fn info(&mut self, args: Arguments);
  /// Log some debug information.
  fn debug(&mut self, args: Arguments);
  /// Log some warnings.
  fn warn(&mut self, args: Arguments);
  /// Log some errors.
  fn error(&mut self, args: Arguments);
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StdoutLogger;

impl Logger for StdoutLogger {
  fn info(&mut self, args: Arguments) {
    println!("\x1b[90m{} \x1b[34m> {}\x1b[0m", now(), args);
  }

  fn debug(&mut self, args: Arguments) {
    println!("\x1b[90m{} \x1b[34m> {}\x1b[0m", now(), args);
  }

  fn warn(&mut self, args: Arguments) {
    println!("\x1b[90m{} \x1b[33m> {}\x1b[0m", now(), args);
  }

  fn error(&mut self, args: Arguments) {
    println!("\x1b[90m{} \x1b[31m> {}\x1b[0m", now(), args);
  }
}

pub fn now() -> String {
  let t = Local::now();
    
  format!("{month:0>2}/{day:0>2}/{year} {hour:0>2}:{min:0>2}:{secs:0>2}:{nsecs:0>9}",
          month = t.month(),
          day = t.day(),
          year = t.year(),
          hour = t.hour(),
          min = t.minute(),
          secs = t.second(),
          nsecs = t.nanosecond())
}

#[macro_export]
macro_rules! info {
  ($logger:expr, $s:expr $(, $r:tt)*) => {{
    use $crate::logger::Logger;
    $logger.info(format_args!($s $(, $r)*));
  }}
}

#[macro_export]
macro_rules! debug {
  ($logger:expr, $s:expr $(, $r:tt)*) => {
    use $crate::logger::Logger;
    $logger.debug(format_args!($s $(, $r)*));
  }
}

#[macro_export]
macro_rules! warn {
  ($logger:expr, $s:expr $(, $r:tt)*) => {
    use $crate::logger::Logger;
    $logger.warn(format_args!($s, $($r)*));
  }
}

#[macro_export]
macro_rules! error {
  ($logger:expr, $s:expr, $($r:tt),*) => {
    use $crate::logger::Logger;
    $logger.error(format_args!($s, $($r),*));
  }
}
