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

#[macro_export]
macro_rules! impl_load_json {
  ($ty_:ty, $desc:expr) => {
    impl $crate::sys::res::helpers::TyDesc for $ty_ {
      const TY_DESC: &'static str = $desc;
    }

    impl ::warmy::Load for $ty_ {
      type Key = ::warmy::PathKey;
    
      type Error = $crate::sys::res::encoding::JSONError;
    
      fn load(key: Self::Key, _: &mut ::warmy::Storage) -> Result<::warmy::Loaded<Self>, Self::Error> {
        let path = key.as_path();
    
        $crate::sys::res::helpers::load_with::<Self, _, _>(path, || {
          let fh = ::std::fs::File::open(path).map_err(|_| $crate::sys::res::encoding::JSONError::FileNotFound(path.to_owned()))?;
    
          let decoded: $ty_ = ::serde_json::from_reader(fh).map_err($crate::sys::res::encoding::JSONError::JSON)?;
    
          Ok(decoded.into())
        })
      }
    
      impl_reload_passthrough!();
    }
  }
}
