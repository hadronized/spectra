//! Viewport renderer.

use luminance::context::GraphicsContext;
use luminance::tess::{Mode, Tess};
use luminance::shader::program::Program;
use std::path::Path;
use warmy::Res;

pub struct ViewportRenderer {
  quad: Tess<()>,
  shader: Res<Program<(), (), ()>>
}

impl ViewportRenderer {
  fn new<Ctx>(gfx_ctx: &mut Ctx, path: &Path) -> Self where Ctx: GraphicsContext {
    let quad = Tess::attributeless(gfx_ctx, Mode::TriangleFan, 4);
    let shader = unimplemented!();

    ViewportRenderer {
      quad,
      shader
    }
  }
}
