use luminance::{Dim2, Flat, Framebuffer, Mode, Pipe, Pipeline, RGBA32F, RawTexture, RenderCommand,
                ShadingCommand, Tess, TessRender, Texture, Unit, Uniform};

use compositor::{Compositor, Screen};
use id::Id;
use scene::Scene;
use shader::Program;

pub type Texture2D<A> = Texture<Flat, Dim2, A>;

const FORWARD_SOURCE: &'static Uniform<Unit> = &Uniform::new(0);

pub struct Forward<'a> {
  program: Id<'a, Program>,
  quad: Tess,
  w: u32,
  h: u32
}

impl<'a> Forward<'a> {
  pub fn new(w: u32, h: u32, scene: &mut Scene<'a>) -> Self {
    let program = get_id!(scene, "spectra/compositors/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap();

    Forward {
      program: program,
      quad: Tess::attributeless(Mode::TriangleStrip, 4),
      w: w,
      h: h
    }
  }
}

impl<'a, 'b> Compositor<'a, 'b, &'a Texture2D<RGBA32F>> for Forward<'b> {
  fn composite(&'a self, scene: &'a mut Scene<'b>, source: &'a Texture2D<RGBA32F>) -> Screen<'a> {
    let program = scene.get_by_id(&self.program).unwrap();
    let back_fb = Framebuffer::default((self.w, self.h));
    let textures: &[&RawTexture] = &[source];
    let tess_render = TessRender::one_whole(&self.quad);

    Pipeline::new(&back_fb, [0., 0., 0., 0.], textures, &[], vec![
      Pipe::empty()
        .uniforms(&[FORWARD_SOURCE.alter(Unit::new(0))])
        .unwrap(ShadingCommand::new(&program, vec![
          Pipe::new(RenderCommand::new(None, true, vec![
            Pipe::new(tess_render)]))
          ]))
    ]).run();

    Screen::Display
  }
}
