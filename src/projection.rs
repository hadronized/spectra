use linear::{Matrix4, Perspective3};

pub trait Projectable {
  fn projection(&self) -> Matrix4<f32>;
}

pub fn perspective(ratio: f32, fovy: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
  Perspective3::new(ratio, fovy, znear, zfar).to_matrix()
}

