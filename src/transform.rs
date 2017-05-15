use linear::M44;

/// A transform matrix, used to represent transformations of objects in space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform(M44<f32>);

impl From<M44<f32>> for Transform {
  fn from(mat44: M44<f32>) -> Self {
    Transform(mat44)
  }
}

impl From<Transform> for M44<f32> {
  fn from(Transform(transform): Transform) -> Self {
    transform
  }
}

impl From<[[f32; 4]; 4]> for Transform {
  fn from(mat44: [[f32; 4]; 4]) -> Self {
    Transform(mat44.into())
  }
}

impl From<Transform> for [[f32; 4]; 4] {
  fn from(Transform(transform): Transform) -> Self {
    transform.into()
  }
}

/// Class of types that can yield transformation matrices.
pub trait Transformable {
  fn transform(&self) -> Transform;
}

impl Transformable for Transform {
  fn transform(&self) -> Transform {
    *self
  }
}
