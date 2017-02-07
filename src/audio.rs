use alto::{self, SourceTrait};
use std::fs::File;
use std::path::Path;
use vorbis::Decoder;

/// The audio object you can use to interact with the soundtrack.
pub struct Audio<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  /// Length of the track.
  len: f32,
  /// OpenAL source.
  source: &'c mut alto::StreamingSource<'a, 'b>
}

impl<'a, 'b, 'c> Audio<'a, 'b, 'c> where 'a: 'b, 'b: 'c {
  pub fn len(&self) -> f32 {
    self.len
  }

  pub fn cursor(&mut self) -> f32 {
    let source = &mut self.source;

    let c = source.sec_offset().unwrap();

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

    if source.state().unwrap() == alto::SourceState::Playing {
      // pause the OpenAL source
      let _ = source.pause();
      false
    } else {
      // unpause the OpenAL source
      let _ = source.play();
      true
    }
  }

  pub fn open<P, A, F>(track_path: P, f: F) -> A where P: AsRef<Path>, F: FnOnce(Audio) -> A {
    deb!("initializing OpenAL");

    let alto = alto::Alto::load_default().unwrap();
    let al_device = alto.open(None).unwrap();
    let al_ctx = al_device.new_context(None).unwrap();

    // create the required objects to play the soundtrack
    let mut al_buffer = al_ctx.new_buffer().unwrap();
    let mut al_source = al_ctx.new_streaming_source().unwrap();

    info!("loading soundtrack {:?}", track_path.as_ref());

    // FIXME: stream the file instead?
    // load PCM data from the file
    let vorbis_decoder = Decoder::new(File::open(track_path).unwrap()).unwrap();
    let mut pcm_buffer = Vec::new();

    for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
      pcm_buffer.extend(packet.data);
    }

    // fill the OpenAL buffers with the PCM data
    let _ = al_buffer.set_data::<alto::Stereo<_>, _>(&pcm_buffer[..], 44100);

    // compute the length of soundtrack
    let len = (al_buffer.size().unwrap() * 8 / (al_buffer.channels().unwrap() * al_buffer.bits().unwrap())) as f32 / al_buffer.frequency().unwrap() as f32;

    let _ = al_source.queue_buffer(al_buffer);

    let audio = Audio { len: len, source: &mut al_source };

    f(audio)
  }

}
