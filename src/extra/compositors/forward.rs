use luminance::{Dim2, Flat, Mode, RGBA32F, Unit};
use luminance_gl::gl33::{Framebuffer, Pipe, Pipeline, RenderCommand, ShadingCommand, Tess, Texture,
                         Uniform};

use compositor::{Compositor, Screen};
use id::Id;
use scene::Scene;
use shader::Program;

pub type Texture2D<A> = Texture<Flat, Dim2, A>;

const FORWARD_SOURCE: Uniform<Unit> = Uniform::new(0);

pub struct Forward<'a> {
  program: Id<'a, Program>,
  quad: Tess,
  w: u32,
  h: u32
}

impl<'a> Forward<'a> {
  pub fn new(w: u32, h: u32, scene: &mut Scene<'a>) -> Self {
    let program = get_id!(scene, "spectra/compositors/forward.glsl", vec![Uniform::<Unit>::sem("source")]).unwrap();

    // update the texture uniform once and for all
    {
      let program: &Program = &scene.get_by_id(&program).unwrap();
      program.update(&FORWARD_SOURCE, Unit::new(0));
    }

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
    let textures = &[source.into()];

    Pipeline::new(&back_fb, [0., 0., 0., 0.], textures, &[], vec![
      Pipe::new(|_| {}, ShadingCommand::new(&program, vec![
        Pipe::new(|_| {}, RenderCommand::new(None, true, vec![
          Pipe::new(|_|{}, &self.quad)], 1, None))
        ]))
    ]).run();

    Screen::Display
  }
}
