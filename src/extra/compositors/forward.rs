use luminance::{Dim2, Flat, Framebuffer, Mode, Pipe, Pipeline, RGBA32F, RawTexture, RenderCommand,
                ShadingCommand, Tess, TessRender, Texture, Unit, Uniform};

use color::ColorAlpha;
use resource::Res;
use scene::Scene;
use shader::Program;

pub type Texture2D<A> = Texture<Flat, Dim2, A>;

const FORWARD_SOURCE: &'static Uniform<Unit> = &Uniform::new(0);

/// The forward compositor.
///
/// This compositor is a very simple one: it forwards a render to the screen. It can also create an
/// empty render – it basically cleans the screen with the color of your choice.
pub struct Forward {
  framebuffer: Framebuffer<Flat, Dim2, (), ()>,
  program: Res<Program>,
  quad: Tess
}

impl Forward {
  pub fn new(w: u32, h: u32, scene: &mut Scene) -> Self {
    let program = scene.get("spectra/compositors/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap();

    Forward {
      framebuffer: Framebuffer::default((w, h)),
      program: program,
      quad: Tess::attributeless(Mode::TriangleStrip, 4),
    }
  }

  /// Display a texture onto screen.
  pub fn texture_screen(&self, source: &Texture2D<RGBA32F>) {
    let textures: &[&RawTexture] = &[source];
    let tess_render = TessRender::one_whole(&self.quad);

    Pipeline::new(&self.framebuffer, [0., 0., 0., 0.], textures, &[], vec![
      Pipe::empty()
        .uniforms(&[FORWARD_SOURCE.alter(Unit::new(0))])
        .unwrap(ShadingCommand::new(&self.program.borrow(), vec![
          Pipe::new(RenderCommand::new(None, true, vec![
            Pipe::new(tess_render)]))
          ]))
    ]).run();
  }

  /// Render a black screen to screen.
  pub fn black_screen(&self, clear_color: ColorAlpha) {
    Pipeline::new(&self.framebuffer, *clear_color.as_ref(), &[], &[], vec![]).run();
  }
}
