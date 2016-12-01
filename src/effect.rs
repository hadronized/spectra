// TODO: maybe we should move that directly in compositor.rs?
use luminance::{Dim2, Flat};
use luminance::framebuffer::ColorSlot;
use luminance_gl::gl33::{Framebuffer, GL33};
use shader::Program;
use std::marker::PhantomData;

pub trait Effect<'a>: Sized {
  type Input;
  type Output;

  fn apply(&'a mut self, Self::Input) -> Self::Output;

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

  fn apply(&'a mut self, input: Self::Input) -> Self::Output {
    self.back.apply(self.front.apply(input))
  }
}

pub struct BaseEffect<Output> where Output: ColorSlot<GL33, Flat, Dim2> {
  framebuffer: Framebuffer<Flat, Dim2, Output, ()>,
  program: Program
}
