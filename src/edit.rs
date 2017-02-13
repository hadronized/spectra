use luminance::{Dim2, Flat, RGBA32F, Texture};

/// Time.
pub type Time = f32;

/// A clip is an object that implements an infinite visual sequence.
pub struct Clip<'a> {
  act: Box<Fn(Time) -> &'a Texture<Flat, Dim2, RGBA32F> + 'a>
}

impl<'a> Clip<'a> {
  pub fn new<F>(act: F) -> Self where F: 'a + Fn(Time) -> &'a Texture<Flat, Dim2, RGBA32F> {
    Clip {
      act: Box::new(act)
    }
  }
}

/// A cut is an object that slices a `Clip` at an *input time* and *output time*. It is instantiated
/// in a `Track` at a given *instance time*.
pub struct Cut<'a> {
  in_time: Time,
  out_time: Time,
  inst_time: Time,
  clip: &'a Clip<'a>
}

impl<'a> Cut<'a> {
  pub fn new(in_time: Time, out_time: Time, inst_time: Time, clip: &'a Clip<'a>) -> Self {
    Cut {
      in_time: in_time,
      out_time: out_time,
      inst_time: inst_time,
      clip: clip
    }
  }
}

/// A track gathers `Cut`s and its purpose is to be used inside a `Timeline`.
pub struct Track<'a> {
  cuts: Vec<Cut<'a>>
}

/// A timeline gathers tracks used to build up the visual aspect of the demo.
pub struct Timeline<'a> {
  tracks: Vec<Track<'a>>,
}
