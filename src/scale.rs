use linear::Matrix4;
use num::One;
use std::default::Default;
use std::ops::Mul;

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
