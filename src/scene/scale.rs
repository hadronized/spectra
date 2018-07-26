pub use num_traits::One;
use std::default::Default;
use std::ops::Mul;

use linear::M44;
use scene::transform::{Transform, Transformable};

/// Arbritrary scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Scale {
  /// Arbitrary scale along the X, Y and Z axis.
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Scale {
      x: x,
      y: y,
      z: z
    }
  }

  /// Uniform scale along the X, Y and Z axis.
  pub fn uni(x: f32) -> Self {
    Scale {
      x: x,
      y: x,
      z: x
    }
  }
}

impl Default for Scale {
  fn default() -> Self { Scale::new(1., 1., 1.) }
}

impl From<[f32; 3]> for Scale {
  fn from([x, y, z]: [f32; 3]) -> Self {
    Scale {
      x: x,
      y: y,
      z: z
    }
  }
}

impl From<Scale> for [f32; 3] {
  fn from(scale: Scale) -> Self {
    [scale.x, scale.y, scale.z]
  }
}

impl From<Scale> for M44<f32> {
  fn from(scale: Scale) -> Self {
    let mat: M44<f32> = [
      [scale.x,      0.,      0., 0.],
      [     0., scale.y,      0., 0.],
      [     0.,      0., scale.z, 0.],
      [     0.,      0.,      0., 1.]
    ].into();

    mat.into()
  }
}

impl Transformable for Scale {
  fn transform(&self) -> Transform {
    let m: M44<_> = (*self).into();

    m.into() 
  }
}

impl Mul for Scale {
  type Output = Scale;

  fn mul(self, rhs: Self) -> Self::Output {
    Scale {
      x: self.x * rhs.x,
      y: self.y * rhs.y,
      z: self.z * rhs.z
    }
  }
}

impl One for Scale {
  fn one() -> Self {
    Scale::new(1., 1., 1.)
  }
}

