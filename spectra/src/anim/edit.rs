use std::collections::HashMap;

/// Time.
pub type Time = f64;

pub struct Clip<'a, A> where A: 'a {
  gen_node: Box<Fn(Time) -> A + 'a>
}

impl<'a, A> Clip<'a, A> {
  pub fn new<F>(f: F) -> Self where F: 'a + Fn(Time) -> A {
    Clip {
      gen_node: Box::new(f)
    }
  }
}

/// A cut is an object that slices a `Clip` at an *input time* and *output time*. It is instantiated
/// in a `Track` at a given *instance time*.
#[derive(Clone)]
pub struct Cut<'a, 'b, A> where A: 'a, 'a: 'b {
  pub in_time: Time,
  pub out_time: Time,
  pub inst_time: Time,
  pub clip: &'b Clip<'a, A>
}

impl<'a, 'b, A> Cut<'a, 'b, A> where A :'a, 'a: 'b {
  pub fn new(in_time: Time, out_time: Time, inst_time: Time, clip: &'b Clip<'a, A>) -> Self {
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
pub struct Track<'a, 'b, A> where A: 'a, 'a: 'b {
  cuts: Vec<Cut<'a, 'b, A>>
}

impl<'a, 'b, A> Track<'a, 'b, A> where A: 'a, 'a: 'b {
  pub fn new() -> Self {
    Track {
      cuts: Vec::new()
    }
  }

  pub fn add_cut(&mut self, cut: Cut<'a, 'b, A>) {
    self.cuts.push(cut);
  }
}

impl<'a, 'b, 'c, A> From<&'c [Cut<'a, 'b, A>]> for Track<'a, 'b, A> where A: Clone {
  fn from(cuts: &'c [Cut<'a, 'b, A>]) -> Self {
    Track {
      cuts: cuts.to_vec()
    }
  }
}

/// A timeline gathers tracks used to build up the visual aspect of the demo.
pub struct Timeline<'a, 'b, A> where A: 'a, 'a: 'b {
  tracks: Vec<Track<'a, 'b, A>>,
  overlaps: Vec<Overlap<'a, A>>
}

impl<'a, 'b, A> Timeline<'a, 'b, A> where A: 'a, 'a: 'b {
  pub fn new() -> Self {
    Timeline {
      tracks: Vec::new(),
      overlaps: Vec::new()
    }
  }

  /// Turn a TimelineManifest into a Timeline by providing a mapping between clips’ names and real
  /// clips.
  pub fn from_manifest(manifest: &TimelineManifest, mapping: &HashMap<String, &'b Clip<'a, A>>) -> Self {
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

  pub fn add_track(&mut self, track: Track<'a, 'b, A>) {
    self.tracks.push(track);
  }

  pub fn add_overlap(&mut self, overlap: Overlap<'a, A>) {
    self.overlaps.push(overlap)
  }

  pub fn play(&self, t: Time) -> Played<A> {
    let mut active_nodes = Vec::new();

    // populate the active nodes
    for track in &self.tracks {
      for cut in &track.cuts {
        if cut.inst_time <= t && t <= cut.inst_time + cut.dur() {
          active_nodes.push((cut.clip.gen_node)(t));
        }
      }
    }

    // apply overlap if needed
    match active_nodes.len() {
      0 => Played::Inactive,
      1 => active_nodes.pop().map(Played::Resolved).unwrap_or(Played::Inactive),
      _ => {
        // we need to seek for an overlap here because we have strictly more than one node in hands
        self.find_overlap(t).map(|overlap| {
          Played::Resolved((overlap.fold)(active_nodes))
        }).unwrap_or(Played::NoOverlap)
      }
    }
  }

  /// Find an active overlap at the given time.
  fn find_overlap(&self, t: Time) -> Option<&Overlap<A>> {
    self.overlaps.iter().find(|x| x.inst_time <= t && t <= x.inst_time + x.dur)
  }
}

/// Informational value giving hints about how a timeline has played.
pub enum Played<A> {
  /// The timeline has correctly resolved everything.
  Resolved(A),
  /// There are active `Track`s but no overlap to fold them.
  NoOverlap,
  /// No active `Track`s.
  Inactive
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TimelineManifest {
  pub tracks: Vec<TrackManifest>
}

impl_load_json!(TimelineManifest, "timeline manifest");

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

/// An overlap is a fold consuming clips’ outputs down to a single one. It’s used whenever two cuts
/// overlap and need to be merged into a single one. It can be used for styling effect or transitions.
pub struct Overlap<'a, A> {
  pub inst_time: Time,
  pub dur: Time,
  pub fold: Box<Fn(Vec<A>) -> A + 'a>,
}

impl<'a, A> Overlap<'a, A> {
  pub fn new<F>(inst_time: Time, dur: Time, f: F) -> Self where F: 'a + Fn(Vec<A>) -> A {
    Overlap {
      inst_time: inst_time,
      dur: dur,
      fold: Box::new(f)
    }
  }
}
