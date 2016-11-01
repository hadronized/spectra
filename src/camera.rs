use nalgebra::{Quaternion, Rotate, ToHomogeneous, Unit, UnitQuaternion, Vector3};
use std::default::Default;
use std::f32::consts::FRAC_PI_4;

use projection::{Projectable, perspective};
use transform::{Axis, M44, Orientation, Position, Transformable, Translation, X_AXIS, Y_AXIS,
                Z_AXIS, translation_matrix};

pub struct Camera<P> {
  pub position: Position,
  pub orientation: Orientation,
  pub properties: P
}

impl<P> Camera<P> {
  pub fn new(position: Position, orientation: Orientation, properties: P) -> Self {
    Camera {
      position: position,
      orientation: orientation,
      properties: properties
    }
  }
}

impl<P> Default for Camera<P> where P: Default {
  fn default() -> Self {
    Camera::new(Position::new(0., 0., 0.),
                Orientation::from_unit_value_unchecked(Quaternion::from_parts(1., Vector3::new(0., 0., 0.))),
                P::default())
  }
}

impl<P> Transformable for Camera<P> {
  fn transform(&self) -> M44 {
    let m = self.orientation.to_rotation_matrix().to_homogeneous() * translation_matrix(-self.position);
    m.as_ref().clone()
  }
}

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
}

impl Default for Freefly {
  fn default() -> Self {
    Self::new()
  }
}

impl Projectable for Freefly {
  fn projection(&self) -> M44 {
    perspective(self.ratio, self.fovy, self.znear, self.zfar)
  }
}

impl Camera<Freefly> {
  pub fn mv(&mut self, dir: Translation) {
    let p = &self.properties;
    let axis = dir * Axis::new(p.strafe_sens, p.upward_sens, p.forward_sens);
    let v = self.orientation.inverse_rotate(&axis);

    self.position -= v;
  }

  pub fn look_around(&mut self, dir: Translation) {
    let p = &self.properties;

    fn orient(axis: &Axis, phi: f32) -> Orientation {
      UnitQuaternion::from_axisangle(Unit::new(&axis), phi)
    }

    self.orientation = orient(&Y_AXIS, p.yaw_sens * dir.y) * self.orientation;
    self.orientation = orient(&X_AXIS, p.pitch_sens * dir.x) * self.orientation;
    self.orientation = orient(&Z_AXIS, p.roll_sens * dir.z) * self.orientation;
  }
}
