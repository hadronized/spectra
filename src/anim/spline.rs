pub use splines::{Key, Interpolate, Interpolation};
use serde::de::DeserializeOwned;
use serde_json::{Error as JsonError, from_reader};
use splines;
use std::error;
use std::fmt;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use warmy::methods::JSON;

use sys::res::{FSKey, Load, Loaded, Storage};
use sys::res::helpers::{TyDesc, load_with};

#[derive(Clone, Debug)]
struct Spline<T>(splines::Spline<T>);

impl<T> Deref for Spline<T> {
  type Target = splines::Spline<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for Spline<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> TyDesc for Spline<T> {
  const TY_DESC: &'static str = "spline";
}

impl<C, T> Load<C, JSON> for Spline<T> where T: 'static + SplineDeserializerAdapter {
  type Key = FSKey;

  type Error = SplineError;

  fn load(key: Self::Key, _: &mut Storage<C>, _: &mut C) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_path();

    load_with::<Self, _, _, _>(path, move || {
      let file = File::open(path).map_err(|_| SplineError::FileNotFound(path.to_owned()))?;
      let keys: Vec<Key<T::Deserialized>> = from_reader(file).map_err(SplineError::ParseFailed)?;
      let spline = splines::Spline::from_iter(keys.into_iter().map(|key|
        Key::new(key.t, T::from_deserialized(key.value), key.interpolation)
      ));

      Ok(Spline(spline).into())
    })
  }

  impl_reload_passthrough!(C, JSON);
}

#[derive(Debug)]
pub enum SplineError {
  FileNotFound(PathBuf),
  ParseFailed(JsonError)
}

impl fmt::Display for SplineError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SplineError::FileNotFound(ref path) => write!(f, "file not found: {}", path.display()),
      SplineError::ParseFailed(ref e) => write!(f, "failed to parse: {}", e)
    }
  }
}

impl error::Error for SplineError {}

/// Spline deserializer adapter used to deserialize splines which keys’ values types don’t directly
/// implement deserialization.
pub trait SplineDeserializerAdapter {
  type Deserialized: DeserializeOwned;

  fn from_deserialized(de: Self::Deserialized) -> Self;
}
// 
// impl SplineDeserializerAdapter for f32 {
//   type Deserialized = Self;
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de
//   }
// }
// 
// impl<T> SplineDeserializerAdapter for V2<T> where T: BaseFloat + DeserializeOwned {
//   type Deserialized = [T; 2];
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de.into()
//   }
// }
// 
// impl<T> SplineDeserializerAdapter for V3<T> where T: BaseFloat + DeserializeOwned {
//   type Deserialized = [T; 3];
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de.into()
//   }
// }
// 
// impl<T> SplineDeserializerAdapter for V4<T> where T: BaseFloat + DeserializeOwned {
//   type Deserialized = [T; 4];
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de.into()
//   }
// }
// 
// impl<T> SplineDeserializerAdapter for Quat<T> where T: BaseFloat + DeserializeOwned {
//   type Deserialized = [T; 4];
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de.into()
//   }
// }
// 
// impl SplineDeserializerAdapter for Scale {
//   type Deserialized = [f32; 3];
// 
//   fn from_deserialized(de: Self::Deserialized) -> Self {
//     de.into()
//   }
// }
// 
// /// Iterator over spline keys.
// pub struct SplineIterator<'a, T> where T: 'a {
//   anim_param: &'a Spline<T>,
//   i: usize
// }
// 
// impl<'a, T> Iterator for SplineIterator<'a, T> {
//   type Item = &'a Key<T>;
// 
//   fn next(&mut self) -> Option<Self::Item> {
//     let r = self.anim_param.0.get(self.i);
// 
//     if let Some(_) = r {
//       self.i += 1;
//     }
// 
//     r
//   }
// }
// 
// impl<'a, T> IntoIterator for &'a Spline<T> {
//   type Item = &'a Key<T>;
//   type IntoIter = SplineIterator<'a, T>;
// 
//   fn into_iter(self) -> Self::IntoIter {
//     SplineIterator {
//       anim_param: self,
//       i: 0
//     }
//   }
// }
// 
// /// Keys that can be interpolated in between. Implementing this trait is required to perform
// /// sampling on splines.
// pub trait Interpolate: Copy {
//   /// Linear interpolation.
//   fn lerp(a: Self, b: Self, t: f32) -> Self;
//   /// Cubic hermite interpolation.
//   ///
//   /// Default to `Self::lerp`.
//   fn cubic_hermite(_: (Self, f32), a: (Self, f32), b: (Self, f32), _: (Self, f32), t: f32) -> Self {
//     Self::lerp(a.0, b.0, t)
//   }
// }
// 
// impl Interpolate for f32 {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     a * (1. - t) + b * t
//   }
// 
//   fn cubic_hermite(x: (Self, f32), a: (Self, f32), b: (Self, f32), y: (Self, f32), t: f32) -> Self {
//     cubic_hermite(x, a, b, y, t)
//   }
// }
// 
// impl Interpolate for V2<f32> {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     a.lerp(b, t)
//   }
// 
//   fn cubic_hermite(x: (Self, f32), a: (Self, f32), b: (Self, f32), y: (Self, f32), t: f32) -> Self {
//     cubic_hermite(x, a, b, y, t)
//   }
// }
// 
// impl Interpolate for V3<f32> {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     a.lerp(b, t)
//   }
// 
//   fn cubic_hermite(x: (Self, f32), a: (Self, f32), b: (Self, f32), y: (Self, f32), t: f32) -> Self {
//     cubic_hermite(x, a, b, y, t)
//   }
// }
// 
// impl Interpolate for V4<f32> {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     a.lerp(b, t)
//   }
// 
//   fn cubic_hermite(x: (Self, f32), a: (Self, f32), b: (Self, f32), y: (Self, f32), t: f32) -> Self {
//     cubic_hermite(x, a, b, y, t)
//   }
// }
// 
// impl Interpolate for Quat<f32> {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     a.nlerp(b, t)
//   }
// }
// 
// impl Interpolate for Scale {
//   fn lerp(a: Self, b: Self, t: f32) -> Self {
//     let av = V3::new(a.x, a.y, a.z);
//     let bv = V3::new(b.x, b.y, b.z);
//     let r = av.lerp(bv, t);
// 
//     Scale::new(r.x, r.y, r.z)
//   }
// }
// 
// // Default implementation of Interpolate::cubic_hermit.
// pub fn cubic_hermite<T>(x: (T, f32), a: (T, f32), b: (T, f32), y: (T, f32), t: f32) -> T
//     where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Div<f32, Output = T> {
//   // time stuff
//   let t2 = t * t;
//   let t3 = t2 * t;
//   let two_t3 = 2. * t3;
//   let three_t2 = 3. * t2;
// 
//   // tangents
//   let m0 = (b.0 - x.0) / (b.1 - x.1);
// 	let m1 = (y.0 - a.0) / (y.1 - a.1);
// 
//   a.0 * (two_t3 - three_t2 + 1.) + m0 * (t3 - 2. * t2 + t) + b.0 * (-two_t3 + three_t2) + m1 * (t3 - t2)
// }
// 
// // Normalize a time ([0;1]) given two control points.
// pub fn normalize_time<T>(t: f32, cp: &Key<T>, cp1: &Key<T>) -> f32 {
//   (t - cp.t) / (cp1.t - cp.t)
// }
// 
// // Find the lower control point corresponding to a given time.
// fn search_lower_cp<T>(cps: &[Key<T>], t: f32) -> Option<usize> {
//   let mut i = 0;
//   let len = cps.len();
// 
//   if len < 2 {
//     return None;
//   }
// 
//   loop {
//     let cp = &cps[i];
//     let cp1 = &cps[i+1];
// 
//     if t >= cp1.t {
//       if i >= len - 2 {
//         return None;
//       }
// 
//       i += 1;
//     } else if t < cp.t {
//       if i == 0 {
//         return None;
//       }
// 
//       i -= 1;
//     } else {
//       break; // found
//     }
//   }
// 
//   Some(i)
// }
