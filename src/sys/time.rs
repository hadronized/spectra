use std::time::Instant;
use std::fmt;

/// Absolute time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time(f64);

impl Time {
  pub fn as_f64(self) -> f64 {
    self.0
  }
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.0.fmt(f)
  }
}

/// Monotonic time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monotonic(Instant);

impl Monotonic {
  pub fn now() -> Self {
    Monotonic(Instant::now())
  }

  pub fn elapsed_secs(&self) -> Time {
    let dur = self.0.elapsed();
    let secs = dur.as_secs() as f64;
    let nanos = dur.subsec_nanos() as f64;

    Time(secs + nanos * 1e-9)
  }
}
