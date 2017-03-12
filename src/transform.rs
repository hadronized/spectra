use linear::UnitQuaternion;

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

pub fn translation_matrix(v: Translation) -> Matrix4<f32> {
  Matrix4::new(
    1., 0., 0., v.x,
    0., 1., 0., v.y,
    0., 0., 1., v.z,
    0., 0., 0.,  1.,
  )
}
