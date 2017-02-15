//! Default shaders.

use luminance::{M44, Uniform};
use std::ops::Deref;

use shader::Program;
use resource::Res;
use scene::Scene;

pub type ColorUniform = Uniform<[f32; 3]>;
pub type Mat44Uniform = Uniform<M44>;

pub const DEFAULT_2D_COLOR: ColorUniform = Uniform::new(0);

pub struct DefaultProgram2D(Res<Program>);

impl Deref for DefaultProgram2D {
  type Target = Res<Program>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DefaultProgram2D {
  pub fn new(scene: &mut Scene) -> Option<Self> {
    scene.get("spectra/default_2d.glsl", vec![DEFAULT_2D_COLOR.sem("color")]).map(DefaultProgram2D)
  }
}

macro_rules! uniforms {
  ($sem_index:expr => $name:ident : $t:ty) => {
    pub const $name : Uniform<$t> = Uniform::new($sem_index);
  };

  ($($name:ident : $t:ty),*) => {
    $(
      uniforms!{0 => $name : $t}
    )*
  };
}

uniforms!{
  DEFAULT_3D_PROJ: M44,
  DEFAULT_3D_VIEW: M44,
  DEFAULT_3D_INST: M44
}

pub struct DefaultProgram3D(Res<Program>);

impl Deref for DefaultProgram3D {
  type Target = Res<Program>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DefaultProgram3D {
  pub fn new(scene: &mut Scene) -> Option<Self> {
    scene.get("spectra/default_3d.glsl", vec![
              DEFAULT_3D_PROJ.sem("proj"),
              DEFAULT_3D_VIEW.sem("view"),
              DEFAULT_3D_INST.sem("inst")])
      .map(DefaultProgram3D)
  }
}
