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
pub fn load_with<T, A, E, F>(
  path: &Path,
  loader: F
) -> Result<A, E>
where F: FnOnce() -> Result<A, E>,
      T: TyDesc {
  info!("loading {} {}", T::TY_DESC, path.display());

  let start_time = Instant::now();
  let r = loader();
  let t = start_time.elapsed();
  let ns = t.as_secs() as f64 * 1e9 + t.subsec_nanos() as f64;
  let (pretty_time, suffix) = load_time(ns);

  if let Ok(_) = r {
    info!("loaded {} {}: {:.3}{}", T::TY_DESC, path.display(), pretty_time, suffix);
  } else {
    err!("fail to load {} {}: {:.3}{}", T::TY_DESC, path.display(), pretty_time, suffix);
  }

  r
}

/// Default reload helper (pass-through).
///
/// This function will log any error that happens.
///
/// Whatever the result of the computation, this function returns it untouched.
pub fn reload_passthrough<C, M, T>(
  _: &T,
  key: T::Key,
  storage: &mut Storage<C>,
  ctx: &mut C
) -> Result<T, T::Error>
where T: Load<C, M>,
      T::Key: Clone + fmt::Debug {
  let r = T::load(key.clone(), storage, ctx);

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
  ($ctx_ty:ty, $method:ty) => {
    fn reload(&self, key: Self::Key, storage: &mut $crate::sys::res::Storage<$ctx_ty>, ctx: &mut $ctx_ty) -> Result<Self, Self::Error> {
      $crate::sys::res::helpers::reload_passthrough::<_, $method, _>(self, key, storage, ctx)
    }
  };

  ($ctx_ty:ty) => {
    impl_reload_passthrough!($ctx_ty, ());
  };
}
