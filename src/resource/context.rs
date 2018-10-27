//! Resource context.

use chrono::{Datelike, Local, Timelike};
use std::fmt::Arguments;

/// Trait use to log activity.
pub trait Logger {
  /// Log some information.
  fn info(args: Arguments);
  /// Log some debug information.
  fn debug(args: Arguments);
  /// Log some warnings.
  fn warn(args: Arguments);
  /// Log some errors.
  fn error(args: Arguments);
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StdoutLogger;

impl Logger for StdoutLogger {
  fn info(args: Arguments) {
    println!("\x1b[90m{} \x1b[34m> {}\x1b[0m", now(), args);
  }

  fn debug(args: Arguments) {
    println!("\x1b[90m{} \x1b[34m> {}\x1b[0m", now(), args);
  }

  fn warn(args: Arguments) {
    println!("\x1b[90m{} \x1b[33m> {}\x1b[0m", now(), args);
  }

  fn error(args: Arguments) {
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
