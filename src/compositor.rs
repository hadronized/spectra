use luminance::{Dim2, Flat, RGBA32F, Texture};
use std::marker::PhantomData;

use resource::Res;
use scene::Scene;

pub enum Screen {
  /// Screen display.
  ///
  /// This shrinks the output to the display and consumes it, displaying the content to the user.
  Display,
  /// Captured output screen.
  ///
  /// This variant is used to express the situation when you don’t want the engine to output to the
  /// screen but instead in a capture texture. You can then do whatever you want with that texture,
  /// like dumping it as a video frame-by-frame capture, stream it or analyse it. Whatever the fuck
  /// you want.
  Capture(Res<Texture<Flat, Dim2, RGBA32F>>),
}

pub trait Compositor<Input> {
  fn composite(&self, &mut Scene, Input) -> Screen;
}
