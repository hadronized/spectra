use std::fmt;
use std::path::Path;
use std::time::Instant;
use warmy::{Load, Storage};

/// A trait that gives a compile-time string representation of a type – akin to the intrinsics
/// core::intrisics::type_name, but without the unsafe / unstable interface, and more customizable
/// as it’s a closed set (not all types implement this trait).
pub trait TyDesc {
  const TY_DESC: &'static str;
}

/// Load helper.
///
/// Call this function whenever you need to load a resource and that you want logged information,
/// such as failures, timing, etc.
pub fn load_with<T, A, F>(
  path: &Path,
  loader: F
) -> A
where F: FnOnce() -> A,
      T: TyDesc {
  info!("loading \x1b[0;35m{}\x1b[0m \x1b[1;32m{}\x1b[0m", T::TY_DESC, path.display());

  let start_time = Instant::now();
  let a = loader();
  let t = start_time.elapsed();
  let ns = t.as_secs() as f64 * 1e9 + t.subsec_nanos() as f64;
  let (pretty_time, suffix) = load_time(ns);

  info!("loaded \x1b[0;35m{}\x1b[0m \x1b[1;32m{}\x1b[0m: \x1b[1;31m{:.3}{}\x1b[0m", T::TY_DESC, path.display(), pretty_time, suffix);

  a
}

/// Default reload helper (pass-through).
///
/// This function will log any error that happens.
///
/// Whatever the result of the computation, this function returns it untouched.
pub fn reload_passthrough<T>(
  _: &T,
  key: T::Key,
  storage: &mut Storage
) -> Result<T, T::Error>
where T: Load,
      T::Key: Clone + fmt::Debug {
  let r = T::load(key.clone(), storage);

  if let Err(ref e) = r {
    err!("cannot reload {:?}: {:#?}", key, e);
  }

  r.map(|x| x.res)
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

#[macro_export]
macro_rules! impl_reload_passthrough {
  () => {
    fn reload(&self, key: Self::Key, storage: &mut Storage) -> Result<Self, Self::Error> {
      $crate::sys::res::helpers::reload_passthrough(self, key, storage)
    }
  }
}
