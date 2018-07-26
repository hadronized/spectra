//! Some default types and impls for common encodings.

use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum JSONError {
  FileNotFound(PathBuf),
  JSON(SerdeError)
}

impl fmt::Display for JSONError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for JSONError {
  fn description(&self) -> &str {
    match *self {
      JSONError::FileNotFound(_) => "file not found",
      JSONError::JSON(_) => "JSON error"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      JSONError::JSON(ref e) => Some(e),
      _ => None
    }
  }
}

/// Implement `Load` for a type that can be deserialized from JSON.
#[macro_export]
macro_rules! impl_load_json {
  ($ty_:ty, $desc:expr) => {
    impl $crate::sys::res::helpers::TyDesc for $ty_ {
      const TY_DESC: &'static str = $desc;
    }

    impl<C> $crate::sys::res::Load<C> for $ty_ {
      type Key = $crate::sys::res::FSKey;

      type Error = $crate::sys::res::encoding::JSONError;

      fn load(key: Self::Key, _: &mut $crate::sys::res::Storage<C>, _: &mut C) -> Result<$crate::sys::res::Loaded<Self>, Self::Error> {
        let path = key.as_path();

        $crate::sys::res::helpers::load_with::<Self, _, _, _>(path, || {
          let fh = ::std::fs::File::open(path).map_err(|_| $crate::sys::res::encoding::JSONError::FileNotFound(path.to_owned()))?;

          let decoded: $ty_ = ::serde_json::from_reader(fh).map_err($crate::sys::res::encoding::JSONError::JSON)?;

          Ok(decoded.into())
        })
      }

      impl_reload_passthrough!(C);
    }
  }
}

/// Implement `Load` for a type that can be indirectly deserialized from JSON by going from a type
/// that implements JSON deserialization.
///
/// This macro will generate the `Load` instance for `$ty_` if `$ty_: From<$intermediate_ty>` holds.
#[macro_export]
macro_rules! impl_load_json_via {
  ($ty_:ty, $intermediate_ty:ty, $desc:expr) => {
    impl $crate::sys::res::helpers::TyDesc for $ty_ {
      const TY_DESC: &'static str = $desc;
    }

    impl<C> $crate::sys::res::Load<C> for $ty_ {
      type Key = $crate::sys::res::FSKey;

      type Error = $crate::sys::res::encoding::JSONError;

      fn load(key: Self::Key, _: &mut $crate::sys::res::Storage<C>, _: &mut C) -> Result<$crate::sys::res::Loaded<Self>, Self::Error> {
        let path = key.as_path();

        $crate::sys::res::helpers::load_with::<Self, _, _>(path, || {
          let fh = ::std::fs::File::open(path).map_err(|_| $crate::sys::res::encoding::JSONError::FileNotFound(path.to_owned()))?;

          let intermediate: $intermediate_ty = ::serde_json::from_reader(fh).map_err($crate::sys::res::encoding::JSONError::JSON)?;
          let decoded: $ty_ = intermediate.into();

          Ok(decoded.into())
        })
      }

      impl_reload_passthrough!(C);
    }
  }
}
