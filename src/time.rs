use std::str::FromStr;
use std::time::Instant;
use std::fmt;

/// Absolute time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time(f64);

impl Time {
  /// Convert into seconds.
  pub fn as_secs(self) -> f64 {
    self.0
  }

  /// Wrap time with a given duration.
  pub fn wrap_around(self, t: Time) -> Self {
    Time(self.0 % t.0)
  }
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.0.fmt(f)
  }
}

impl From<DurationSpec> for Time {
  fn from(spec: DurationSpec) -> Self {
    Time(spec.mins as f64 * 60. + spec.secs as f64)
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

/// A way to specify a duration with minutes and seconds.
///
/// The string format is the following:
///
///   - `MmSs`, if you want minutes (e.g. `3m43s`).
///   - `Ss`, if you only have seconds (e.g. `23s`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurationSpec {
  mins: u8,
  secs: u8
}

impl FromStr for DurationSpec {
  type Err = DurationSpecError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if !s.ends_with('s') {
      if s.ends_with('m') {
        // only minutes
        let mins = s.trim_end_matches('m').parse().map_err(|_| DurationSpecError::CannotParseMinutes)?;

        Ok(DurationSpec { mins, secs: 0 })
      } else {
        Err(DurationSpecError::MissingSecondsSuffix)
      }
    } else if s.contains('m') {
      // the first argument represents minutes, so let’s take them out first
      let mut iter = s.split('m');
      let mins = iter.next().and_then(|x| x.parse().ok()).ok_or(DurationSpecError::CannotParseMinutes)?;
      let secs = iter.next().and_then(|x| x.trim_end_matches('s').parse().ok()).ok_or(DurationSpecError::CannotParseSeconds)?;

      Ok(DurationSpec { mins, secs })
    } else {
      // only seconds
      let secs = s.trim_end_matches('s').parse().map_err(|_| DurationSpecError::CannotParseSeconds)?;

      Ok(DurationSpec { mins: 0, secs })
    }
  }
}

/// Possible error than can occurr while parsing a `DurationSpec` from a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationSpecError {
  MissingSecondsSuffix,
  CannotParseMinutes,
  CannotParseSeconds
}

impl fmt::Display for DurationSpecError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      DurationSpecError::MissingSecondsSuffix => f.write_str("missing the seconds suffix"),
      DurationSpecError::CannotParseMinutes => f.write_str("cannot parse minutes"),
      DurationSpecError::CannotParseSeconds => f.write_str("cannot parse seconds"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_duration_spec() {
    assert_eq!("1s".parse::<DurationSpec>().unwrap(), DurationSpec { mins: 0, secs: 1 });
    assert_eq!("2m".parse::<DurationSpec>().unwrap(), DurationSpec { mins: 2, secs: 0 });
    assert_eq!("3m12s".parse::<DurationSpec>().unwrap(), DurationSpec { mins: 3, secs: 12 });
    assert_eq!("3m12".parse::<DurationSpec>(), Err(DurationSpecError::MissingSecondsSuffix));
  }
}
