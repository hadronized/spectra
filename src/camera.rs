use luminance::M44;
use nalgebra::Rotate;
use std::default::Default;
use std::f32::consts::FRAC_PI_4;

use entity::{Axis, Entity, Translation, X_AXIS, Y_AXIS, Z_AXIS};
use projection::perspective;

pub struct Freefly {
  // sensitivities
  pub yaw_sens: f32,
  pub pitch_sens: f32,
  pub roll_sens: f32,
  pub forward_sens: f32,
  pub strafe_sens: f32,
  pub upward_sens: f32,
  // projection
  pub ratio: f32,
  pub fovy: f32,
  // clipping
  pub znear: f32,
  pub zfar: f32,
}

impl Freefly {
  pub fn new() -> Self {
    Freefly {
      yaw_sens: 0.01,
      pitch_sens: 0.01,
      roll_sens: 0.01,
      forward_sens: 0.1,
      strafe_sens: 0.1,
      upward_sens: 0.1,
      ratio: 4. / 3.,
      fovy: FRAC_PI_4,
      znear: 0.1,
      zfar: 10.,
    }
  }

  pub fn projection_matrix(&self) -> M44 {
    perspective(self.ratio, self.fovy, self.znear, self.zfar)
  }
}

impl Default for Freefly {
  fn default() -> Self {
    Self::new()
  }
}

impl Entity<Freefly> {
  pub fn mv(&mut self, dir: Translation) {
    let cam = &self.object;
    let axis = dir * Axis::new(cam.strafe_sens, cam.upward_sens, cam.forward_sens);
    let v = self.transform.orientation.inverse_rotate(&axis);

    self.transform = self.transform.translate(v);
  }

  pub fn look_around(&mut self, dir: Translation) {
    let cam = &self.object;

    self.transform = self.transform.orient(Y_AXIS * dir.y, cam.yaw_sens);
    self.transform = self.transform.orient(X_AXIS * dir.x, cam.pitch_sens);
    self.transform = self.transform.orient(Z_AXIS * dir.z, cam.roll_sens);
  }
}
