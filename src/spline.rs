use std::f32::consts;
use std::ops::{Add, Div, Mul, Sub};
use nalgebra::{UnitQuaternion, Vector2, Vector3, Vector4};

pub use serde_json::ser::to_string;

/// Time used as sampling type in splines.
pub type Time = f32;

/// A spline control point.
///
/// This type associates a value at a given type. It also contains an interpolation object used to
/// determine how to interpolate values on the segment defined by this key and the next one.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Key<T> {
  /// Time at which the `Key` should be reached.
  pub t: Time,
  /// Actual value.
  pub value: T,
  /// Interpolation mode.
  pub interpolation: Interpolation
}

impl<T> Key<T> {
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

/// Spline curve used to provide interpolation between control points (keys).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Spline<T> {
  keys: Vec<Key<T>>,
}

impl<T> Spline<T> {
  pub fn new(mut cps: Vec<Key<T>>) -> Self {
    cps.sort_by(|k0, k1| k0.t.partial_cmp(&k1.t).unwrap());

    Spline {
      keys: cps
    }
  }
}

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

/// Implement this trait if your type is a key you want to sample with.
pub trait Interpolate: Copy {
  /// Linear interpolation.
  fn lerp(a: Self, b: Self, t: Time) -> Self;
  /// Cubic hermite interpolation.
  fn cubic_hermite(_: (Self, Time), a: (Self, Time), b: (Self, Time), _: (Self, Time), t: Time) -> Self {
    Self::lerp(a.0, b.0, t)
  }
}

impl Interpolate for f32 {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    lerp(a, b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for Vector2<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    lerp(a, b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for Vector3<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    lerp(a, b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for Vector4<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    lerp(a, b, t)
  }

  fn cubic_hermite(x: (Self, Time), a: (Self, Time), b: (Self, Time), y: (Self, Time), t: Time) -> Self {
    cubic_hermite(x, a, b, y, t)
  }
}

impl Interpolate for UnitQuaternion<f32> {
  fn lerp(a: Self, b: Self, t: Time) -> Self {
    a * UnitQuaternion::new(&(UnitQuaternion::new(&a.quaternion().conjugate()) * b).quaternion().powf(t))
  }
}

// Default implementation of Interpolate::lerp.
fn lerp<T>(a: T, b: T, t: Time) -> T where T: Add<Output = T> + Mul<Time, Output = T> {
  a * (1. - t) + b * t
}

// Default implementation of Interpolate::cubic_hermit.

fn cubic_hermite<T>(x: (T, Time), a: (T, Time), b: (T, Time), y: (T, Time), t: Time) -> T
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

/// Samplers can sample `Spline` by providing a `Time`. They should be mutable so that they can
/// maintain an internal state for optimization purposes.
#[derive(Copy, Clone, Default)]
pub struct Sampler {
  /// Playback cursor – gives the lower control point index of the current portion of the curve
  /// we’re sampling at.
  cursor: usize
}

impl Sampler {
  pub fn new() -> Self {
    Sampler {
      cursor: 0
    }
  }

  /// Sample a spline. If `random_sampling` is set, random sampling is generally faster than
  /// continuous sampling. Though, if you use continuous sampling, set `random_sampling` to `false`
	/// for max speed performance.
  pub fn sample<T>(&mut self, t: Time, param: &Spline<T>, random_sampling: bool) -> Option<T>
      where T: Interpolate {
    let i = if random_sampling {
      binary_search_lower_cp(&param.keys, t)
    } else {
      let i = around_search_lower_cp(&param.keys, self.cursor, t);

      // if we’ve found the index, replace the cursor to speed up next searches
      if let Some(cursor) = i {
        self.cursor = cursor;
      }

      i
    };

    let i = match i {
      Some(i) => i,
      None => return None
    };

    let cp0 = &param.keys[i];

    match cp0.interpolation {
      Interpolation::Step(threshold) => {
        let cp1 = &param.keys[i+1];
        Some(if t < threshold { cp0.value } else { cp1.value })
      },
      Interpolation::Linear => {
        let cp1 = &param.keys[i+1];
        let nt = normalize_time(t, cp0, cp1);

        Some(Interpolate::lerp(cp0.value, cp1.value, nt))
      },
      Interpolation::Cosine => {
        let cp1 = &param.keys[i+1];
        let nt = normalize_time(t, cp0, cp1);
        let cos_nt = (1. - f32::cos(nt * consts::PI)) * 0.5;

        Some(Interpolate::lerp(cp0.value, cp1.value, cos_nt))
      },
      Interpolation::CatmullRom => {
        // We need at least four points for Catmull Rom; ensure we have them, otherwise, return
        // None.
        if i == 0 || i >= param.keys.len() - 2 {
          None
        } else {
          let cp1 = &param.keys[i+1];
          let cpm0 = &param.keys[i-1];
          let cpm1 = &param.keys[i+2];
          let nt = normalize_time(t, cp0, cp1);

          Some(Interpolate::cubic_hermite((cpm0.value, cpm0.t), (cp0.value, cp0.t), (cp1.value, cp1.t), (cpm1.value, cpm1.t), nt))
        }
      }
    }
  }
}

// Normalize a time ([0;1]) given two control points.
fn normalize_time<T>(t: Time, cp: &Key<T>, cp1: &Key<T>) -> Time {
  (t - cp.t) / (cp1.t - cp.t)
}

// Find the lower control point corresponding to a given time. Random version.
fn binary_search_lower_cp<T>(cps: &[Key<T>], t: Time) -> Option<usize> {
  let len = cps.len() as i32;
  if len < 2 {
    return None;
  }

  let mut down = 0;
  let mut up = len - 1;

  while down <= up {
    let m = (up + down) / 2;
    if m < 0 || m >= len - 1 {
      return None;
    }

    let cp0 = &cps[m as usize];

    if cp0.t > t {
      up = m-1;
    } else {
      let cp1 = &cps[(m+1) as usize];

      if t >= cp1.t {
        down = m+1;
      } else {
        return Some(m as usize)
      }
    }
  }

  None
}

// Find the lower control point corresponding to a given time. Continuous version. `i` is the last
// known found index.
fn around_search_lower_cp<T>(cps: &[Key<T>], mut i: usize, t: Time) -> Option<usize> {
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
