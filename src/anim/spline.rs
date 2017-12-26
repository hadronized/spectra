use cgmath::{BaseFloat, InnerSpace};
use serde::de::DeserializeOwned;
use serde_json::{Error as JsonError, from_reader};
use std::error::Error;
use std::fmt;
use std::f32::consts;
use std::fs::File;
use std::ops::{Add, Div, Mul, Sub};
use std::path::PathBuf;

use linear::{Scale, Quat, V2, V3, V4};
use sys::resource::{DebugRes, Load, Loaded, PathKey, Store, load_with};

/// Time used as sampling type in splines.
pub type Time = f32;

/// A spline control point.
///
/// This type associates a value at a given time. It also contains an interpolation object used to
/// determine how to interpolate values on the segment defined by this key and the next one.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Key<T> {
  /// Time at which the `Key` should be reached.
  pub t: Time,
  /// Actual value.
  pub value: T,
  /// Interpolation mode.
  #[serde(default)]
  pub interpolation: Interpolation
}

impl<T> Key<T> {
  /// Create a new key.
  pub fn new(t: Time, value: T, interpolation: Interpolation) -> Self {
    Key {
      t: t,
      value: value,
      interpolation: interpolation
    }
  }
}

/// Interpolation mode.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Interpolation {
  /// Hold a `Key` until the time passes the normalized step threshold, in which case the next
  /// key is used.
  ///
  /// *Note: if you set the threshold to `0.5`, the first key will be used until the time is half
  /// between the two keys; the second key will be in used afterwards. If you set it to `1.0`, the
  /// first key will be kept until the next key.*
  #[serde(rename = "step")]
  Step(f32),
  /// Linear interpolation between a key and the next one.
  #[serde(rename = "linear")]
  Linear,
  /// Cosine interpolation between a key and the next one.
  #[serde(rename = "cosine")]
  Cosine,
  /// Catmull-Rom interpolation.
  #[serde(rename = "catmull_rom")]
  CatmullRom
}

impl Default for Interpolation {
  /// `Interpolation::Linear` is the default.
  fn default() -> Self {
    Interpolation::Linear
  }
}

/// Spline curve used to provide interpolation between control points (keys).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Spline<T> {
  keys: Vec<Key<T>>,
}

impl<T> Spline<T> {
  /// Create a new spline out of keys. The keys don’t have to be sorted because they’re sorted by
  /// this function.
  pub fn from_keys(mut keys: Vec<Key<T>>) -> Self {
    keys.sort_by(|k0, k1| k0.t.partial_cmp(&k1.t).unwrap());

    Spline {
      keys: keys
    }
  }

  /// Sample a spline at a given time.
  ///
  /// # Return
  ///
  /// `None` if you try to sample a value at a time that has no key associated with. That can also
  /// happen if you try to sample between two keys with a specific interpolation mode that make the
  /// sampling impossible. For instance, `Interpolate::CatmullRom` requires *four* keys. If you’re
  /// near the beginning of the spline or its end, ensure you have enough keys around to make the
  /// sampling.
  pub fn sample(&self, t: Time) -> Option<T> where T: Interpolate {
    let keys = &self.keys;
    let i = search_lower_cp(keys, t);

    let i = match i {
      Some(i) => i,
      None => return None
    };

    let cp0 = &keys[i];

    match cp0.interpolation {
      Interpolation::Step(threshold) => {
        let cp1 = &keys[i+1];
        let nt = normalize_time(t, cp0, cp1);
        Some(if nt < threshold { cp0.value } else { cp1.value })
      },
      Interpolation::Linear => {
        let cp1 = &keys[i+1];
        let nt = normalize_time(t, cp0, cp1);

        Some(Interpolate::lerp(cp0.value, cp1.value, nt))
      },
      Interpolation::Cosine => {
        let cp1 = &keys[i+1];
        let nt = normalize_time(t, cp0, cp1);
        let cos_nt = (1. - f32::cos(nt * consts::PI)) * 0.5;

        Some(Interpolate::lerp(cp0.value, cp1.value, cos_nt))
      },
      Interpolation::CatmullRom => {
        // We need at least four points for Catmull Rom; ensure we have them, otherwise, return
        // None.
        if i == 0 || i >= keys.len() - 2 {
          None
        } else {
          let cp1 = &keys[i+1];
          let cpm0 = &keys[i-1];
          let cpm1 = &keys[i+2];
          let nt = normalize_time(t, cp0, cp1);

          Some(Interpolate::cubic_hermite((cpm0.value, cpm0.t), (cp0.value, cp0.t), (cp1.value, cp1.t), (cpm1.value, cpm1.t), nt))
        }
      }
    }
  }

  /// Sample a spline at a given time with clamping.
  ///
  /// # Return
  ///
  /// If you sample before the first key or after the last one,
  /// return the first key or the last one, respectively.
  ///
  /// # Panic
  ///
  /// This function panics if you have no key.
  pub fn clamped_sample(&self, t: Time) -> T where T: Interpolate {
    let first = self.keys.first().unwrap();
    let last = self.keys.last().unwrap();

    if t <= first.t {
      return first.value;
    } else if t >= last.t {
      return last.value;
    }

    self.sample(t).unwrap()
  }
}

impl<T> DebugRes for Spline<T> {
  const TYPE_DESC: &'static str = "spline";
}

impl<T> Load for Spline<T> where T: 'static + SplineDeserializerAdapter {
  type Key = PathKey;

  type Error = SplineError;

  fn load(key: Self::Key, _: &mut Store) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_path();

    load_with::<Self, _, _>(path, move || {
      let file = File::open(path).map_err(|_| SplineError::FileNotFound(path.to_owned()))?;
      let keys: Vec<Key<T::Deserialized>> = from_reader(file).map_err(SplineError::ParseFailed)?;

      Ok(Spline::from_keys(keys.into_iter().map(|key|
        Key::new(key.t, T::from_deserialized(key.value), key.interpolation)
      ).collect()).into())
    })
  }

  impl_reload_passthrough!();
}

#[derive(Debug)]
pub enum SplineError {
  FileNotFound(PathBuf),
  ParseFailed(JsonError)
}

impl fmt::Display for SplineError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for SplineError {
  fn description(&self) -> &str {
    match *self {
      SplineError::FileNotFound(_) => "file not found",
      SplineError::ParseFailed(_) => "parse failed"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      SplineError::ParseFailed(ref e) => Some(e),
      _ => None
    }
  }
}

/// Spline deserializer adapter used to deserialize splines which keys’ values types don’t directly
/// implement deserialization.
pub trait SplineDeserializerAdapter {
  type Deserialized: DeserializeOwned;

  fn from_deserialized(de: Self::Deserialized) -> Self;
}

impl SplineDeserializerAdapter for f32 {
  type Deserialized = Self;

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de
  }
}

impl<T> SplineDeserializerAdapter for V2<T> where T: BaseFloat + DeserializeOwned {
  type Deserialized = [T; 2];

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de.into()
  }
}

impl<T> SplineDeserializerAdapter for V3<T> where T: BaseFloat + DeserializeOwned {
  type Deserialized = [T; 3];

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de.into()
  }
}

impl<T> SplineDeserializerAdapter for V4<T> where T: BaseFloat + DeserializeOwned {
  type Deserialized = [T; 4];

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de.into()
  }
}

impl<T> SplineDeserializerAdapter for Quat<T> where T: BaseFloat + DeserializeOwned {
  type Deserialized = [T; 4];

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de.into()
  }
}

impl SplineDeserializerAdapter for Scale {
  type Deserialized = [f32; 3];

  fn from_deserialized(de: Self::Deserialized) -> Self {
    de.into()
  }
}

/// Iterator over spline keys.
pub struct SplineIterator<'a, T> where T: 'a {
  anim_param: &'a Spline<T>,
  i: usize
}

impl<'a, T> Iterator for SplineIterator<'a, T> {
  type Item = &'a Key<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let r = self.anim_param.keys.get(self.i);

    if let Some(_) = r {
      self.i += 1;
    }

    r
  }
}

impl<'a, T> IntoIterator for &'a Spline<T> {
  type Item = &'a Key<T>;
  type IntoIter = SplineIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    SplineIterator {
      anim_param: self,
      i: 0
    }
  }
}

/// Keys that can be interpolated in between. Implementing this trait is required to perform
/// sampling on splines.
pub trait Interpolate: Copy {
  /// Linear interpolation.
  fn lerp(a: Self, b: Self, t: Time) -> Self;
  /// Cubic hermite interpolation.
  ///
  /// Default to `Self::lerp`.
  fn cubic_hermite(_: (Self, Time), a: (Self, Time), b: (Self, Time), _: (Self, Time), t: Time) -> Self {
    Self::lerp(a.0, b.0, t)
  }
}

impl Interpolate for f32 {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * (1. - t) + b * t
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for V2<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a.lerp(b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for V3<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a.lerp(b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for V4<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a.lerp(b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for Quat<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a.nlerp(b, t)
  }
}

impl Interpolate for Scale {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    let av = V3::new(a.x, a.y, a.z);
    let bv = V3::new(b.x, b.y, b.z);
    let r = av.lerp(bv, t);

    Scale::new(r.x, r.y, r.z)
  }
}

// Default implementation of Interpolate::cubic_hermit.
pub fn cubic_hermite<T>(x: (T, Time), a: (T, Time), b: (T, Time), y: (T, Time), t: Time) -> T
    where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Time, Output = T> + Div<Time, Output = T> {
  // time stuff
  let t2 = t * t;
  let t3 = t2 * t;
  let two_t3 = 2. * t3;
  let three_t2 = 3. * t2;

  // tangents
  let m0 = (b.0 - x.0) / (b.1 - x.1);
	let m1 = (y.0 - a.0) / (y.1 - a.1);

  a.0 * (two_t3 - three_t2 + 1.) + m0 * (t3 - 2. * t2 + t) + b.0 * (-two_t3 + three_t2) + m1 * (t3 - t2)
}

// Normalize a time ([0;1]) given two control points.
pub fn normalize_time<T>(t: Time, cp: &Key<T>, cp1: &Key<T>) -> Time {
  (t - cp.t) / (cp1.t - cp.t)
}

// Find the lower control point corresponding to a given time.
fn search_lower_cp<T>(cps: &[Key<T>], t: Time) -> Option<usize> {
  let mut i = 0;
  let len = cps.len();

  if len < 2 {
    return None;
  }

  loop {
    let cp = &cps[i];
    let cp1 = &cps[i+1];

    if t >= cp1.t {
      if i >= len - 2 {
        return None;
      }

      i += 1;
    } else if t < cp.t {
      if i == 0 {
        return None;
      }

      i -= 1;
    } else {
      break; // found
    }
  }

  Some(i)
}
