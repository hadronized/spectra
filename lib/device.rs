use openal::al;
use openal::alc;
use std::fs::File;
use std::path::Path;
use std::mem;
use vorbis::Decoder;

/// The device is responsible of managing synchronization and audio playback by providing a playback
/// cursor to a closure used as demo.
///
/// You shouldn’t have more than one Device per program.
pub struct Device {
  /// Length of the demo (seconds).
  length: f32,
  /// OpenAL device.
  al_device: alc::Device,
  /// OpenAL context.
  al_ctx: alc::Context,
  /// OpenAL buffer.
  al_buffer: al::Buffer,
  /// OpenAL source.
  al_source: al::Source
}

impl Device {
  pub fn new<P>(track_path: P) -> Self where P: AsRef<Path> {
    // initialising OpenAL
    let al_device = alc::Device::open(None).unwrap();
    let al_ctx = al_device.create_context(&[]).unwrap();
    al_ctx.make_current();

    // create the required objects to play the soundtrack
    let al_buffer = al::Buffer::gen();
    let al_source = al::Source::gen();

    // load PCM data from the file
    let vorbis_decoder = Decoder::new(File::open(track_path).unwrap()).unwrap();
    let mut pcm_buffer = Vec::new();

    for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
      pcm_buffer.extend(packet.data);
    }

    // fill the OpenAL buffers with the PCM data
    unsafe { al_buffer.buffer_data(al::Format::Stereo16, &pcm_buffer, 44100) };
    al_source.queue_buffer(&al_buffer);

    // compute the length of soundtrack
    let l = (al_buffer.get_size() * 8 / (al_buffer.get_channels() * al_buffer.get_bits())) as f32 / al_buffer.get_frequency() as f32;
    Device {
      length: l,
      al_device: al_device,
      al_ctx: al_ctx,
      al_buffer: al_buffer,
      al_source: al_source
    }
  }

  /// Playback cursor in seconds.
  pub fn playback_cursor(&self) -> f32 {
    let cursor = self.al_source.get_sec_offset();

    // loop the device if we hit the end of the demo
    if cursor > self.length {
      self.al_source.rewind();
      0.
    } else {
      cursor
    }
  }

  // FIXME: [debug]
  /// [debug] Move the cursor around. Expect the input to be normalized.
  pub fn set_cursor(&mut self, t: f32) {
    assert!(t >= 0. && t <= 1.);
    self.al_source.set_sec_offset(t * self.length);
  }

  pub fn playback_length(&self) -> f32 {
    self.length
  }

  pub fn toggle(&mut self) {
    if self.al_source.is_playing() {
      // pause the OpenAL source
      self.al_source.pause();
    } else {
      // unpause the OpenAL source
      self.al_source.play();
    }
  }
}

impl Drop for Device {
  fn drop(&mut self) {
    drop(&mut self.al_buffer);
    drop(&mut self.al_source);
    drop(&mut self.al_ctx);

    let dummy = unsafe { mem::uninitialized() };
    let _ = mem::replace(&mut self.al_device, dummy).close();
  }
}
