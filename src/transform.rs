use linear::Matrix4;

pub type Transform = Matrix4<f32>;
/// Class of types that can yield transformation matrices.
pub trait Transformable {
  fn transform(&self) -> Transform;
}

impl Transformable for Matrix4<f32> {
  fn transform(&self) -> Transform {
    self.clone()
  }
}
