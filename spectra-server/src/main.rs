extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate spectra;

mod mode;
mod msg;

use serde_json::de::from_str;
use spectra::render::framebuffer::Framebuffer2D;
use spectra::render::shader::program::{Program, ProgramKey};
use spectra::render::texture::TextureImage;
use spectra::sys::ignite::{Action, GraphicsContext, Ignite, Key, Surface, WindowEvent, WindowOpt};
use spectra::sys::res;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use mode::Mode;

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
                let _ = sx.send(msg);
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
  let back_buffer = Framebuffer2D::back_buffer(ignite.surface().size());
  let clear_color = [0.8, 0.5, 0.5, 1.];

  let mut mode = Mode::default();
  let mut store: res::Store<Ignite> = res::Store::new(res::StoreOpt::default().set_root("data")).expect("resource store creation");

  // for shader toying
  let shadertoy_quad = Tess::attributeless(tess::Mode::TriangleFan, 0, 4);

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
        msg::Msg::Close => break 'l,

        // we’re asked to load a texture at a given position
        msg::Msg::LoadTexture(path) => {
          let _ = store.get::<_, TextureImage>(&res::FSKey::new(&path), &mut ignite);
        }

        // we’re asked to load and use a shader program in fullscreen mode
        msg::Msg::ShaderToy(name) => {
          mode = Mode::ShaderToy(ProgramKey::new(&name));
        }
      }
    }

    // perform the render
    let _t = ignite.time();

    match mode {
      Mode::Empty => {
        ignite.surface().pipeline_builder().pipeline(&back_buffer, clear_color, |_, _| {});
      },

      Mode::ShaderToy(ref key) => {
        let program = store.get::<_, Program<(), (), ()>>(key, &mut ignite);

        if let Ok(ref program) = program {
        }
      }
    }

    ignite.surface().swap_buffers();
  }
}
