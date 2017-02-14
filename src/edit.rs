use luminance::{Dim2, Flat, RGBA32F, Texture};

/// Time.
pub type Time = f32;

/// A clip is an object that implements a visual sequence.
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
#[derive(Clone)]
pub struct Cut<'a> {
  pub in_time: Time,
  pub out_time: Time,
  pub inst_time: Time,
  pub clip: &'a Clip<'a>
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
#[derive(Clone)]
pub struct Track<'a> {
  cuts: Vec<Cut<'a>>
}

impl<'a> Track<'a> {
  pub fn new() -> Self {
    Track {
      cuts: Vec::new()
    }
  }

  pub fn add_cut(&mut self, cut: Cut<'a>) {
    self.cuts.push(cut);
  }
}

impl<'a, 'b> From<&'b [Cut<'a>]> for Track<'a> {
  fn from(cuts: &'b [Cut<'a>]) -> Self {
    Track {
      cuts: cuts.to_vec()
    }
  }
}

/// A timeline gathers tracks used to build up the visual aspect of the demo.
#[derive(Clone)]
pub struct Timeline<'a> {
  tracks: Vec<Track<'a>>,
}

impl<'a> Timeline<'a> {
  pub fn new() -> Self {
    Timeline {
      tracks: Vec::new()
    }
  }

  pub fn add_track(&mut self, track: Track<'a>) {
    self.tracks.push(track);
  }
}

impl<'a, 'b> From<&'b [Track<'a>]> for Timeline<'a> {
  fn from(tracks: &'b [Track<'a>]) -> Self {
    Timeline {
      tracks: tracks.to_vec()
    }
  }
}

