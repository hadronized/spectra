use luminance::M44;
use nalgebra::Perspective3;

pub fn perspective(ratio: f32, fovy: f32, znear: f32, zfar: f32) -> M44 {
  *Perspective3::new(ratio, fovy, znear, zfar).to_matrix().as_ref()
}

