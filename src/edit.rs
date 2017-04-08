use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use compositing::Node;
use resource::{Load, LoadError, Result, ResCache};

/// Time.
pub type Time = f64;

pub struct Clip<'a, 'b> where 'a: 'b {
  gen_node: Box<Fn(Time) -> Node<'a> + 'b>
}

impl<'a, 'b> Clip<'a, 'b> {
  pub fn new<F>(f: F) -> Self where F: 'b + Fn(Time) -> Node<'a> {
    Clip {
      gen_node: Box::new(f)
    }
  }
}

/// A cut is an object that slices a `Clip` at an *input time* and *output time*. It is instantiated
/// in a `Track` at a given *instance time*.
#[derive(Clone)]
pub struct Cut<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  pub in_time: Time,
  pub out_time: Time,
  pub inst_time: Time,
  pub clip: &'c Clip<'a, 'b>
}

impl<'a, 'b, 'c> Cut<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  pub fn new(in_time: Time, out_time: Time, inst_time: Time, clip: &'c Clip<'a, 'b>) -> Self {
    assert!(in_time <= out_time);

    Cut {
      in_time: in_time,
      out_time: out_time,
      inst_time: inst_time,
      clip: clip
    }
  }

  /// Duration of the cut.
  pub fn dur(&self) -> Time {
    self.out_time - self.in_time
  }
}

/// A track gathers `Cut`s and its purpose is to be used inside a `Timeline`.
#[derive(Clone)]
pub struct Track<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  cuts: Vec<Cut<'a, 'b, 'c>>
}

impl<'a, 'b, 'c> Track<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  pub fn new() -> Self {
    Track {
      cuts: Vec::new()
    }
  }

  pub fn add_cut(&mut self, cut: Cut<'a, 'b, 'c>) {
    self.cuts.push(cut);
  }
}

impl<'a, 'b, 'c, 'd> From<&'d [Cut<'a, 'b, 'c>]> for Track<'a, 'b, 'c> {
  fn from(cuts: &'d [Cut<'a, 'b, 'c>]) -> Self {
    Track {
      cuts: cuts.to_vec()
    }
  }
}

/// A timeline gathers tracks used to build up the visual aspect of the demo.
#[derive(Clone)]
pub struct Timeline<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  tracks: Vec<Track<'a, 'b, 'c>>,
}

impl<'a, 'b, 'c> Timeline<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  pub fn new() -> Self {
    Timeline {
      tracks: Vec::new(),
    }
  }

  /// Turn a TimelineManifest into a Timeline by providing a mapping between clips’ names and real
  /// clips.
  pub fn from_manifest(manifest: &TimelineManifest, mapping: &HashMap<String, &'c Clip<'a, 'b>>) -> Self {
    let mut timeline = Self::new();

    for track_manifest in &manifest.tracks {
      let mut track = Track::new();

      for cut_manifest in &track_manifest.cuts {
        let in_time = cut_manifest.in_time;
        let out_time = cut_manifest.out_time;
        let inst_time = cut_manifest.inst_time;

        if let Some(clip) = mapping.get(&cut_manifest.clip).cloned() {
          track.add_cut(Cut::new(in_time, out_time, inst_time, clip));
        } else {
          warn!("the clip {:?} doesn’t exist", cut_manifest.clip);
        }
      }

      timeline.add_track(track);
    }

    timeline
  }

  pub fn add_track(&mut self, track: Track<'a, 'b, 'c>) {
    self.tracks.push(track);
  }

  pub fn play(&self, t: Time) -> Vec<Node<'a>> {
    let mut active_nodes = Vec::new();

    for track in &self.tracks {
      for cut in &track.cuts {
        if cut.inst_time <= t && t <= cut.inst_time + cut.dur() {
          active_nodes.push((cut.clip.gen_node)(t));
        }
      }
    }

    active_nodes
  }
}

impl<'a, 'b, 'c, 'd> From<&'d [Track<'a, 'b, 'c>]> for Timeline<'a, 'b, 'c> {
  fn from(tracks: &'d [Track<'a, 'b, 'c>]) -> Self {
    Timeline {
      tracks: tracks.to_vec(),
    }
  }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TimelineManifest {
  pub tracks: Vec<TrackManifest>
}

impl Load for TimelineManifest {
  type Args = ();

  const TY_STR: &'static str = "edit";

  fn load<P>(path: P, _: &mut ResCache, _: Self::Args) -> Result<Self> where P: AsRef<Path> {
    let path = path.as_ref();

    info!("loading timeline: {:?}", path);

    let file = File::open(path).map_err(|e| LoadError::FileNotFound(path.to_path_buf(), format!("{:?}", e)))?;
    from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))
  }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TrackManifest {
  pub cuts: Vec<CutManifest>
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct CutManifest {
  pub in_time: Time,
  pub out_time: Time,
  pub inst_time: Time,
  pub clip: String
}
