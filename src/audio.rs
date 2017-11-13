use alto::{self, Source};
use std::fs::File;
use std::path::Path;
use vorbis::Decoder;

/// The audio object you can use to interact with the soundtrack.
pub struct Audio {
  /// Length of the track.
  len: f32,
  /// OpenAL source.
  source: alto::StreamingSource
}

impl Audio {
  pub fn len(&self) -> f32 {
    self.len
  }

  pub fn cursor(&mut self) -> f32 {
    let source = &mut self.source;

    let c = source.sec_offset();

    // loop the device if we hit the end of the demo
    if c > self.len {
      let _ = source.rewind();
      0.
    } else {
      c
    }
  }

  pub fn set_cursor(&mut self, t: f32) {
    assert!(t >= 0. && t <= 1.);
    let _ = self.source.set_sec_offset(t * self.len);
  }

  pub fn play(&mut self) {
    let _ = self.source.play();
  }

  pub fn pause(&mut self) {
    let _ = self.source.pause();
  }

  pub fn toggle(&mut self) -> bool {
    let source = &mut self.source;

    if source.state() == alto::SourceState::Playing {
      // pause the OpenAL source
      let _ = source.pause();
      false
    } else {
      // unpause the OpenAL source
      let _ = source.play();
      true
    }
  }

  // FIXME: return the object instead
  pub fn from_track_path<P>(track_path: P) -> Self where P: AsRef<Path> {
    deb!("initializing alto");

    let alto = alto::Alto::load_default().unwrap();
    let al_device = alto.open(None).unwrap();
    let ctx = al_device.new_context(None).unwrap();

    info!("loading soundtrack {:?}", track_path.as_ref());

    // FIXME: stream the file instead?
    // load PCM data from the file
    let vorbis_decoder = Decoder::new(File::open(track_path).unwrap()).unwrap();
    let mut pcm_buffer = Vec::new();

    for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
      pcm_buffer.extend(packet.data);
    }

    // create the required objects to play the soundtrack
    let buffer = ctx.new_buffer::<alto::Stereo<i16>, _>(&pcm_buffer[..], 44100).unwrap();
    let mut source = ctx.new_streaming_source().unwrap();

    // compute the length of soundtrack
    let len = (buffer.size() * 8 / (buffer.channels() * buffer.bits())) as f32 / buffer.frequency() as f32;

    let _ = source.queue_buffer(buffer);

    Audio { len, source }
  }
}
