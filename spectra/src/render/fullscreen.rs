//! Fullscreen objects.

use luminance::context::GraphicsContext;
use luminance::tess::{Mode, Tess};
use std::ops::Deref;

/// A fullscreen quad.
///
/// The vertices must be provided CCW and with a triangle fan patch distribution.
pub struct Quad {
  tess: Tess<()>
}

impl Quad {
  pub fn new<C>(ctx: &mut C) -> Self where C: GraphicsContext {
    let tess = Tess::attributeless(ctx, Mode::TriangleFan, 4);
    Self { tess }
  }
}

impl Deref for Quad {
  type Target = Tess<()>;

  fn deref(&self) -> &Self::Target {
    &self.tess
  }
}
