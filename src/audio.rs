use alto::{self, SourceTrait};
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::spawn;
use vorbis::Decoder;

/// The device is responsible of managing synchronization and audio playback by providing a playback
/// cursor to a closure used as demo.
///
/// You shouldn’t have more than one Device per program.
pub struct Device {
  /// Playback length.
  len: f32,
  /// Sender used to send requests to audio thread.
  req_sx: SyncSender<Request>,
  /// Receiver used to receive responses from the audio thread.
  resp_rx: Receiver<Response>
}

enum Request {
  Toggle,
  PlaybackLength,
  PlaybackCursor,
  SetCursor(f32)
}

enum Response {
  PlaybackLength(f32),
  PlaybackCursor(f32)
}

impl Device {
  pub fn new<P>(track_path: P) -> Self where P: AsRef<Path> {
    info!("loading soundtrack {:?}", track_path.as_ref());

    // FIXME: stream the file instead?
    // load PCM data from the file
    let vorbis_decoder = Decoder::new(File::open(track_path).unwrap()).unwrap();
    let mut pcm_buffer = Vec::new();

    for packet in vorbis_decoder.into_packets().map(Result::unwrap) {
      pcm_buffer.extend(packet.data);
    }

    // create a channel to send requests to / retrieve them from the audio thread
    let (req_sx, req_rx) = sync_channel(0);
    // create a channel to receive responses / to send responses from the audio thread
    let (mut resp_sx, resp_rx) = sync_channel(0);

    info!("starting audio thread");

    let _ = spawn(move || {
      deb!("initializing OpenAL");

      let alto = alto::Alto::load_default().unwrap();
      let al_device = alto.open(None).unwrap();
      let al_ctx = al_device.new_context(None).unwrap();

      // create the required objects to play the soundtrack
      let mut al_buffer = al_ctx.new_buffer().unwrap();
      let mut al_source = al_ctx.new_streaming_source().unwrap();

      // fill the OpenAL buffers with the PCM data
      let _ = al_buffer.set_data::<alto::Stereo<_>, _>(&pcm_buffer[..], 44100);

      // compute the length of soundtrack
      let l = (al_buffer.size().unwrap() * 8 / (al_buffer.channels().unwrap() * al_buffer.bits().unwrap())) as f32 / al_buffer.frequency().unwrap() as f32;

      let _ = al_source.queue_buffer(al_buffer);

      while let Ok(req) = req_rx.recv() {
        dispatch_request(&mut al_source, l, req, &mut resp_sx);
      }

      info!("leaving the audio thread");
    });

    // get the length
    let _ = req_sx.send(Request::PlaybackLength);
    
    let len = match resp_rx.recv() {
      Ok(Response::PlaybackLength(len)) => len,
      _ => unimplemented!()
    };

    Device {
      len: len,
      req_sx: req_sx,
      resp_rx: resp_rx
    }
  }

  pub fn toggle(&self) {
    let _ = self.req_sx.send(Request::Toggle);
  }

  pub fn playback_len(&self) -> f32 {
    self.len
  }

  pub fn playback_cursor(&self) -> f32 {
    let _ = self.req_sx.send(Request::PlaybackCursor);
    
    match self.resp_rx.recv() {
      Ok(Response::PlaybackCursor(c)) => c,
      _ => unimplemented!()
    }
  }

  pub fn set_playback_cursor(&self, t: f32) {
    let _ = self.req_sx.send(Request::SetCursor(t));
  }
}

fn dispatch_request(source: &mut alto::StreamingSource, len: f32, req: Request, resp_sx: &mut SyncSender<Response>) {
  match req {
    Request::Toggle => {
      toggle_source(source);
    },

    Request::PlaybackLength => {
      let _ = resp_sx.send(Response::PlaybackLength(len));
    },

    Request::PlaybackCursor => {
      let _ = resp_sx.send(Response::PlaybackCursor(playback_cursor_source(source, len)));
    },

    Request::SetCursor(t) => {
      set_playback_cursor_source(source, t, len);
    }
  }
}

fn toggle_source(source: &mut alto::StreamingSource) {
  if source.state().unwrap() == alto::SourceState::Playing {
    // pause the OpenAL source
    let _ = source.pause();
  } else {
    // unpause the OpenAL source
    let _ = source.play();
  }
}

fn playback_cursor_source(source: &mut alto::StreamingSource, len: f32) -> f32 {
  let cursor = source.sec_offset().unwrap();

  // loop the device if we hit the end of the demo
  if cursor > len {
    let _ = source.rewind();
    0.
  } else {
    cursor
  }
}

pub fn set_playback_cursor_source(source: &mut alto::StreamingSource, t: f32, len: f32) {
  assert!(t >= 0. && t <= 1.);
  let _ = source.set_sec_offset(t * len);
}
