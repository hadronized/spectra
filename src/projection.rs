use serde::Deserialize;
use std::f32::consts::FRAC_PI_4;

use linear::{Matrix4, Perspective3};

pub trait Projectable {
  fn projection(&self) -> Matrix4<f32>;
}

pub fn perspective(ratio: f32, fovy: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
  Perspective3::new(ratio, fovy, znear, zfar).to_matrix()
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Perspective {
  #[serde(default = "def_ratio")]
  pub ratio: f32,
  #[serde(default = "def_fovy")]
  pub fovy: f32,
  #[serde(default = "def_znear")]
  pub znear: f32,
  #[serde(default = "def_zfar")]
  pub zfar: f32,
  
}

impl Perspective {
  pub fn new() -> Self {
    Perspective {
      ratio: def_ratio(),
      fovy: def_fovy(),
      znear: def_znear(),
      zfar: def_zfar(),
    }
  }
}

impl Default for Perspective {
  fn default() -> Self {
    Self::new()
  }
}

impl Projectable for Perspective {
  fn projection(&self) -> Matrix4<f32> {
    perspective(self.ratio, self.fovy, self.znear, self.zfar)
  }
}

fn def_ratio() -> f32 { 4. / 3. }
fn def_fovy() -> f32 { FRAC_PI_4 }
fn def_znear() -> f32 { 0.1 }
fn def_zfar() -> f32 { 10. }
