//! Default shaders.

use luminance::M44;
use luminance_gl::gl33::Uniform;
use std::ops::Deref;

use id::Id;
use shader::Program;
use scene::Scene;

pub type ColorUniform = Uniform<[f32; 3]>;
pub type Mat44Uniform = Uniform<M44>;

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
    get_id!(scene, "spectra/default_2d.glsl", vec![ColorUniform::sem("color")]).map(DefaultProgram2D)
  }
}

pub const DEFAULT_3D_PROJ: Mat44Uniform = Uniform::new(0);
pub const DEFAULT_3D_VIEW: Mat44Uniform = Uniform::new(1);
pub const DEFAULT_3D_INST: Mat44Uniform = Uniform::new(2);

pub struct DefaultProgram3D<'a>(Id<'a, Program>);

impl<'a> Deref for DefaultProgram3D<'a> {
  type Target = Id<'a, Program>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> DefaultProgram3D<'a> {
  pub fn get_from(scene: &mut Scene<'a>) -> Option<Self> {
    get_id!(scene, "spectra/default_3d.glsl", vec![
              Mat44Uniform::sem("proj"),
              Mat44Uniform::sem("view"),
              Mat44Uniform::sem("inst")])
      .map(DefaultProgram3D)
  }
}
