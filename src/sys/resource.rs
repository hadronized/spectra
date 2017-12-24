//! Resource system.

use std::path::Path;
use std::time::Instant;

pub use warmy::*;

/// A trait that gives a compile-time string representation of a type – akin to the intrinsics
/// core::intrisics::type_name, but without the unsafe / unstable interface, and more customizable
/// as it’s a closed set (not all types implement this trait).
pub trait DebugRes {
  const TYPE_DESC: &'static str;
}

#[inline]
pub fn load_with<T, A, F>(
  path: &Path,
  loader: F
) -> A
where F: FnOnce() -> A,
      T: DebugRes {
  info!("loading \x1b[0;35m{}\x1b[0m \x1b[1;32m{}\x1b[0m", T::TYPE_DESC, path.display());

  let start_time = Instant::now();
  let a = loader();
  let t = start_time.elapsed();
  let ns = t.as_secs() as f64 * 1e9 + t.subsec_nanos() as f64;
  let (pretty_time, suffix) = load_time(ns);

  info!("loaded \x1b[0;35m{}\x1b[0m \x1b[1;32m{}\x1b[0m: \x1b[1;31m{:.3}{}\x1b[0m", T::TYPE_DESC, path.display(), pretty_time, suffix);

  a
}

fn load_time<'a>(ns: f64) -> (f64, &'a str) {
  if ns >= 1e9 {
    (ns * 1e-9, "s")
  } else if ns >= 1e6 {
    (ns * 1e-6, "ms")
  } else if ns >= 1e3 {
    (ns * 1e-3, "μs")
  } else {
    (ns, "ns")
  }
}
