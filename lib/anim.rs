use std::f32::consts;
use std::ops::{Add, Div, Mul, Sub};
use nalgebra::{UnitQuaternion, Vector2, Vector3, Vector4};

pub type Time = f32;

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum Interpolation {
  /// Hold a `Key` until the next one is met.
  Hold,
  /// Linear interpolation between a `Key` and the next one.
  Linear,
  /// Cosine interpolation between a `Key` and the next one.
  Cosine,
  /// Catmull-Rom interpolation.
  CatmullRom
}

#[derive(Debug)]
pub struct AnimParam<T> {
  control_points: Vec<Key<T>>,
}

impl<T> AnimParam<T> {
  pub fn new(mut cps: Vec<Key<T>>) -> Self {
    cps.sort_by(|k0, k1| k0.t.partial_cmp(&k1.t).unwrap());

    AnimParam {
      control_points: cps
    }
  }
}

pub struct AnimParamIterator<'a, T> where T: 'a {
  anim_param: &'a AnimParam<T>,
  i: usize
}

impl<'a, T> Iterator for AnimParamIterator<'a, T> {
  type Item = &'a Key<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let r = self.anim_param.control_points.get(self.i);

    if let Some(_) = r {
      self.i += 1;
    }

    r
  }
}

impl<'a, T> IntoIterator for &'a AnimParam<T> {
  type Item = &'a Key<T>;
  type IntoIter = AnimParamIterator<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    AnimParamIterator {
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
    a * UnitQuaternion::new_with_quaternion((UnitQuaternion::new_with_quaternion(a.quaternion().conjugate()) * b).quaternion().powf(t))
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

/// Samplers can sample `AnimParam` by providing a time. They should be mutable so that they can
/// maintain an internal state for optimization purposes.
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

  /// Sample an animation `param` at `t`. If `random_sampling` is set, random sampling is generally
	/// faster than continuous sampling. Though, if you use continuous sampling, set `random_sampling`
	/// to `false` for max speed performance.
  pub fn sample<T>(&mut self, t: Time, param: &AnimParam<T>, random_sampling: bool) -> Option<T>
      where T: Interpolate {
    let i = if random_sampling {
      binary_search_lower_cp(&param.control_points, t)
    } else {
      let i = around_search_lower_cp(&param.control_points, self.cursor, t);

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

    let cp0 = &param.control_points[i];

    match cp0.interpolation {
      Interpolation::Hold => Some(cp0.value),
      Interpolation::Linear => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp0, cp1);

        Some(Interpolate::lerp(cp0.value, cp1.value, nt))
      },
      Interpolation::Cosine => {
        let cp1 = &param.control_points[i+1];
        let nt = normalize_time(t, cp0, cp1);
        let cos_nt = (1. - f32::cos(nt * consts::PI)) * 0.5;

        Some(Interpolate::lerp(cp0.value, cp1.value, cos_nt))
      },
      Interpolation::CatmullRom => {
        // We need at least four points for Catmull Rom; ensure we have them, otherwise, return
        // None.
        if i == 0 || i >= param.control_points.len() - 2 {
          None
        } else {
          let cp1 = &param.control_points[i+1];
          let cpm0 = &param.control_points[i-1];
          let cpm1 = &param.control_points[i+2];
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
fn binary_search_lower_cp<T>(cps: &Vec<Key<T>>, t: Time) -> Option<usize> {
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
fn around_search_lower_cp<T>(cps: &Vec<Key<T>>, mut i: usize, t: Time) -> Option<usize> {
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
    } else {
      if t < cp.t {
        if i == 0 {
          return None;
        }

        i -= 1;
      } else {
        break; // found
      }
		}
  }

  Some(i)
}

// FIXME: not sure we need mutability here, because it would lead into unreproductible effects
/// Continuous value.
///
/// This type wraps a `A` as a function of time `f32`. It has a simple semantic: `at`, giving the
/// value at the wished time.
pub struct Cont<'a, A> {
  closure: Box<FnMut(f32) -> A + 'a>
}

impl<'a, A> Cont<'a, A> {
  pub fn new<F>(f: F) -> Self where F: 'a + FnMut(f32) -> A {
    Cont {
      closure: Box::new(f)
    }
  }

  /// Turn a set of discret values that happen at given moments into a continuous step value.
  pub fn from_discrete(def: A, mut moments: Vec<(f32, A)>) -> Self where A: 'a + Clone {
    moments.sort_by(|_, b| b.0.partial_cmp(&b.0).unwrap());

    Cont {
      closure: Box::new(move |t| {
        moments.binary_search_by(|a| a.0.partial_cmp(&t).unwrap()).ok().map_or(def.clone(), |i| moments[i].1.clone())
      })
    }
  }

  pub fn at(&mut self, t: f32) -> A {
    (self.closure)(t)
  }
}

#[macro_export]
macro_rules! simple_animation {
  ($name:ident, $t:ty, $def:expr, [ $( ($k:expr, $v:expr, $i:expr) ),* ]) => {
    fn $name<'a>() -> Cont<'a, $t> {
      let mut sampler = Sampler::new();
      let keys = AnimParam::new(
        vec![
          $( Key::new($k, $v, $i) ),*
      ]);

      Cont::new(move |t| {
        sampler.sample(t, &keys, true).unwrap_or($def)
      })
    }
  }
}
