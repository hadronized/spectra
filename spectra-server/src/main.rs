extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate spectra;

use serde_json::de::from_str;
use spectra::render::framebuffer::Framebuffer2D;
use spectra::sys::ignite::{Action, GraphicsContext, Ignite, Key, Surface, WindowEvent, WindowOpt};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

mod msg;

fn main() {
  match ignite!(960, 540, WindowOpt::default()) {
    Ok(ignite) => {
      deb!("created ignite");

      let (sx, rx) = mpsc::channel();

      thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:6666").unwrap(); 

        for stream in listener.incoming() {
          let stream = stream.unwrap();

          deb!("stream connected: {:?}", stream);

          for line in  BufReader::new(stream).lines() {
            let line = line.unwrap();

            match from_str::<msg::Msg>(&line) {
              Ok(msg) => {
                deb!("received command: {:?}", msg);
                sx.send(msg);
              }

              Err(e) => err!("wrong command: {}", e)
            }

          }

          deb!("stream disconnected");
        }

      });

      main_loop(ignite, rx);
      deb!("bye");
    }

    Err(e) => {
      err!("cannot create ignite: {}", e);
    }
  }
}

fn main_loop(mut ignite: Ignite, rx_msg: mpsc::Receiver<msg::Msg>) {
  let back_buffer = Framebuffer2D::default(ignite.surface().size());
  let clear_color = [0.8, 0.5, 0.5, 1.];

  'l: loop {
    // read events
    for event in ignite.surface().poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'l,
        _ => ()
      }
    }

    for msg in rx_msg.try_iter() {
      match msg {
        msg::Msg::Close => break 'l
      }
    }

    // perform the render
    let t = ignite.time();

    let surface = ignite.surface();

    surface.pipeline_builder().pipeline(&back_buffer, clear_color, |_, _| {});
    surface.swap_buffers();
  }
}
