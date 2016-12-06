use std::default::Default;

use linear::UnitQuaternion;
use num::One;
use std::ops::Mul;

pub use luminance::linear::M44;
pub use linear::{Matrix4, Vector3};

/// Class of types that can yield transformation matrices.
pub trait Transformable {
  fn transform(&self) -> Matrix4<f32>;
}

impl Transformable for Matrix4<f32> {
  fn transform(&self) -> Matrix4<f32> {
    self.clone()
  }
}

pub type Translation = Vector3<f32>;
pub type Axis = Vector3<f32>;
pub type Position = Vector3<f32>;
pub type Orientation = UnitQuaternion<f32>;

pub const X_AXIS: Axis = Axis { x: 1., y: 0., z: 0. };
pub const Y_AXIS: Axis = Axis { x: 0., y: 1., z: 0. };
pub const Z_AXIS: Axis = Axis { x: 0., y: 0., z: 1. };

/// Arbritrary scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Scale {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Scale {
      x: x,
      y: y,
      z: z
    }
  }

  pub fn uni(x: f32) -> Self {
    Scale {
      x: x,
      y: x,
      z: x
    }
  }

  pub fn to_mat(&self) -> Matrix4<f32> {
    Matrix4::new(
      self.x,     0.,     0., 0.,
          0., self.y,     0., 0.,
          0.,     0., self.z, 0.,
          0.,     0.,     0., 1.
    )
  }
}

impl Default for Scale {
  fn default() -> Self { Scale::new(1., 1., 1.) }
}

impl<'a> From<&'a [f32; 3]> for Scale {
  fn from(slice: &[f32; 3]) -> Self {
    Scale {
      x: slice[0],
      y: slice[1],
      z: slice[2]
    }
  }
}

impl<'a> From<&'a Scale> for [f32; 3] {
  fn from(scale: &Scale) -> Self {
    [scale.x, scale.y, scale.z]
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

pub fn translation_matrix(v: Translation) -> Matrix4<f32> {
  Matrix4::new(
    1., 0., 0., v.x,
    0., 1., 0., v.y,
    0., 0., 1., v.z,
    0., 0., 0.,  1.,
  )
}
