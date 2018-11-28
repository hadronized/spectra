//! Quickly create demoscene applications.

use luminance::framebuffer::Framebuffer;
pub use luminance::pipeline::Builder;
use luminance::texture::{Dim2, Flat};
pub use warmy::Store;
use std::fmt::Debug;

use crate::app::context::Context;
pub use crate::resource::key::Key;
pub use crate::time::Time;

/// Class of demo applications.
///
/// A demo is basically just a single function that takes the current time and display something.
pub trait Demo: Sized {
  /// Context carried around with the demo.
  type Context: Context;

  /// Initialization error that might occur.
  type Error: Sized + Debug;

  /// Initialize the demo with a given store.
  fn init(store: &mut Store<Self::Context, Key>, context: &mut Self::Context) -> Result<Self, Self::Error>;

  /// Resize the demo when the framebuffer gets resized.
  fn resize(&mut self, context: &mut Self::Context, width: u32, height: u32);

  /// Render the demo at a given time. 
  fn render(&mut self, context: &mut Self::Context, t: Time, back_buffer: &Backbuffer, builder: Builder);
}

pub type Backbuffer = Framebuffer<Flat, Dim2, (), ()>;
