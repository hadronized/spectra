use luminance::{Dim2, Flat, RGBA32F};
use luminance::framebuffer::ColorSlot;
use luminance_gl::gl33::{Framebuffer, GL33, Texture};
use std::marker::PhantomData;

use scene::Scene;
use shader::Program;

pub enum Screen<'a> {
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
  Capture(&'a Texture<Flat, Dim2, RGBA32F>),
}

pub trait Compositor<'a, 'b, Input> {
  fn composite(&'a mut self, &'a mut Scene<'b>, Input) -> Screen<'a>;
}

pub trait Effect<'a>: Sized {
  type Input;
  type Output;

  fn apply(&mut self, Self::Input) -> Self::Output;

  fn then<A, E>(self, next: E) -> Then<'a, Self::Input, Self::Output, A, Self, E> where
      E: Effect<'a, Input=Self::Output, Output=A> {
    Then {
      front: self,
      back: next,
      _a: PhantomData
    }
  }
}

pub struct Then<'a, A, B, C, I0, I1> where
    I0: Effect<'a, Input=A, Output=B>,
    I1: Effect<'a, Input=B, Output=C> {
  front: I0,
  back: I1,
  _a: PhantomData<&'a ()>
}

impl<'a, A, B, C, I0, I1> Effect<'a> for Then<'a, A, B, C, I0, I1> where
    I0: Effect<'a, Input=A, Output=B>,
    I1: Effect<'a, Input=B, Output=C> {
  type Input = A;
  type Output = C;

  fn apply(&mut self, input: Self::Input) -> Self::Output {
    self.back.apply(self.front.apply(input))
  }
}

pub struct BaseEffect<Output> where Output: ColorSlot<GL33, Flat, Dim2> {
  framebuffer: Framebuffer<Flat, Dim2, Output, ()>,
  program: Program
}
