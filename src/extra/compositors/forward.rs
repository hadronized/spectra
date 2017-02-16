use luminance::{Dim2, Flat, Framebuffer, Mode, Pipe, Pipeline, RGBA32F, RawTexture, RenderCommand,
                ShadingCommand, Tess, TessRender, Texture, Unit, Uniform};

use compositor::{Compositor, Screen};
use resource::Res;
use scene::Scene;
use shader::Program;

pub type Texture2D<A> = Texture<Flat, Dim2, A>;

const FORWARD_SOURCE: &'static Uniform<Unit> = &Uniform::new(0);

pub struct Forward {
  program: Res<Program>,
  quad: Tess,
  w: u32,
  h: u32
}

impl Forward {
  pub fn new(w: u32, h: u32, scene: &mut Scene) -> Self {
    let program = scene.get("spectra/compositors/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap();

    Forward {
      program: program,
      quad: Tess::attributeless(Mode::TriangleStrip, 4),
      w: w,
      h: h
    }
  }

  pub fn to_screen(&self, source: &Texture2D<RGBA32F>) {
    let back_fb = Framebuffer::default((self.w, self.h));
    let textures: &[&RawTexture] = &[source];
    let tess_render = TessRender::one_whole(&self.quad);

    Pipeline::new(&back_fb, [0., 0., 0., 0.], textures, &[], vec![
      Pipe::empty()
        .uniforms(&[FORWARD_SOURCE.alter(Unit::new(0))])
        .unwrap(ShadingCommand::new(&self.program.borrow(), vec![
          Pipe::new(RenderCommand::new(None, true, vec![
            Pipe::new(tess_render)]))
          ]))
    ]).run();
  }

  pub fn black_screen(&self) {
    let back_fb = Framebuffer::default((self.w, self.h));
    Pipeline::new(&back_fb, [0., 0., 0., 0.], &[], &[], vec![]).run();
  }
}
