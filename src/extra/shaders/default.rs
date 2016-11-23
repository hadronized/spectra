//! Default shaders.

use luminance::Sem;
use luminance_gl::gl33::Uniform;
use std::ops::Deref;

use id::Id;
use shader::Program;
use scene::Scene;

pub type ColorUniform = Uniform<[f32; 4]>;
pub const DEFAULT_2D_COLOR: ColorUniform = Uniform::new(0);

pub struct DefaultProgram2D<'a>(Id<'a, Program>);

impl<'a> Deref for DefaultProgram2D<'a> {
  type Target = Id<'a, Program>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> DefaultProgram2D<'a> {
  pub fn get_from(scene: &mut Scene<'a>) -> Option<Self> {
    get_id!(scene, "default_2d.glsl", vec![ColorUniform::sem("color")]).map(DefaultProgram2D)
  }
}
