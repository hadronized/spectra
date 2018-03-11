use cgmath::{self, Rad};
use std::f32::consts::FRAC_PI_4;

use linear::M44;

pub struct Projection(M44<f32>);

impl AsRef<M44<f32>> for Projection {
  fn as_ref(&self) -> &M44<f32> {
    &self.0
  }
}

impl From<[[f32; 4]; 4]> for Projection {
  fn from(mat44: [[f32; 4]; 4]) -> Self {
    Projection(mat44.into())
  }
}

impl From<Projection> for [[f32; 4]; 4] {
  fn from(Projection(projection): Projection) -> Self {
    projection.into()
  }
}

pub trait Projectable {
  fn projection(&self) -> Projection;
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Perspective {
  #[serde(default = "def_aspect")]
  pub aspect: f32,
  #[serde(default = "def_fovy")]
  pub fovy: f32,
  #[serde(default = "def_znear")]
  pub znear: f32,
  #[serde(default = "def_zfar")]
  pub zfar: f32,
  
}

impl Default for Perspective {
  fn default() -> Self {
    Perspective {
      aspect: def_aspect(),
      fovy: def_fovy(),
      znear: def_znear(),
      zfar: def_zfar(),
    }
  }
}

impl Projectable for Perspective {
  fn projection(&self) -> Projection {
    Projection(cgmath::perspective(Rad(self.fovy), self.aspect, self.znear, self.zfar))
  }
}

fn def_aspect() -> f32 { 4. / 3. }
fn def_fovy() -> f32 { FRAC_PI_4 }
fn def_znear() -> f32 { 0.1 }
fn def_zfar() -> f32 { 10. }
