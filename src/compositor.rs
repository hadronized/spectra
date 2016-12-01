use luminance::{Dim2, Flat, RGBA32F};
use luminance_gl::gl33::Texture;

use effect::Effect;

pub enum Screen<'a> {
  /// Screen display.
  ///
  /// This shrinks the output to the display and consume it, displaying the content to the user.
  Display,
  /// Captured output screen.
  ///
  /// This variant is used to express the situation when you don’t want the engine to output to the
  /// screen but instead in a capture texture. You can then do whatever you want with that texture,
  /// like dumping it as a video frame-by-frame capture, stream it or analyse it. Whatever the fuck
  /// you want.
  Capture(&'a Texture<Flat, Dim2, RGBA32F>),
}

pub trait Compositor<'a, I> {
  fn composite(&'a mut self, input: I) -> Screen;
}

impl<'a, E, I> Compositor<'a, I> for E where E: Effect<'a, Input=I, Output=()> {
  fn composite(&'a mut self, input: I) -> Screen {
    self.apply(input);
    Screen::Display
  }
}
