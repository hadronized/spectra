extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate spectra;
#[cfg(feature = "websocket_server")] extern crate ws;

mod mode;
mod msg;
mod server;

use spectra::luminance::render_state::RenderState;
use spectra::luminance::tess::TessSliceIndex;
use spectra::render::framebuffer::Framebuffer2D;
use spectra::render::shader::program::{Program, ProgramKey};
use spectra::render::texture::TextureImage;
use spectra::sys::ignite::{Action, GraphicsContext, Ignite, Key, Surface, WindowEvent, WindowOpt};
use spectra::sys::res;
use spectra::render::fullscreen::Quad;
use std::sync::mpsc::Receiver;

use msg::Msg;
use mode::Mode;
use server::core::{Server, start_server};

#[cfg(not(feature = "websocket_server"))] use server::tcp::TcpServer;
#[cfg(feature = "websocket_server")] use server::ws::WSServer;

#[cfg(feature = "websocket_server")]
fn get_server() -> impl Server {
  WSServer
}

#[cfg(not(feature = "websocket_server"))]
fn get_server() -> impl Server {
  TcpServer
}

fn main() {
  match ignite!(960, 540, WindowOpt::default()) {
    Ok(ignite) => {
      deb!("created ignite");

      let rx = start_server(get_server());

      main_loop(ignite, rx);
      deb!("bye");
    }

    Err(e) => {
      err!("cannot create ignite: {}", e);
    }
  }
}

fn main_loop(mut ignite: Ignite, rx_msg: Receiver<Msg>) {
  let mut back_buffer = Framebuffer2D::back_buffer(ignite.surface().size());
  let clear_color = [0.8, 0.5, 0.5, 1.];

  let mut mode = Mode::default();
  let mut store: res::Store<Ignite> = res::Store::new(res::StoreOpt::default().set_root("data")).expect("resource store creation");

  // for shader toying
  let shadertoy_quad = Quad::new(ignite.surface());

  'l: loop {
    // read events
    for event in ignite.surface().poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'l,

        WindowEvent::FramebufferSize(w, h) => {
          back_buffer = Framebuffer2D::back_buffer([w as u32, h as u32]);
        }
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

        msg::Msg::EmptyMode => {
          mode = Mode::Empty;
        }

        // we’re asked to load and use a shader program in fullscreen mode
        msg::Msg::ShaderToy(name) => {
          let pkey = ProgramKey::new(&name);

          if let Err(e) = store.get::<_, Program<(), (), ()>>(&pkey, &mut ignite) {
            err!("{:?}", e);
          } else {
            mode = Mode::ShaderToy(pkey);
          }
        }
      }
    }

    // perform the render
    let _t = ignite.time();

    let new_mode = match mode {
      Mode::Empty => {
        ignite.surface().pipeline_builder().pipeline(&back_buffer, clear_color, |_, _| {});
        None
      },

      Mode::ShaderToy(ref key) => {
        let program = store.get::<_, Program<(), (), ()>>(key, &mut ignite);

        if let Ok(ref program) = program {
          ignite.surface().pipeline_builder().pipeline(&back_buffer, clear_color, |_, shd_gate| {
            shd_gate.shade(&program.borrow(), |rdr_gate, _| {
              rdr_gate.render(RenderState::default(), |tess_gate| {
                tess_gate.render(ignite.surface(), shadertoy_quad.slice(..));
              });
            });
          });

          None
        } else {
          // fail to use shader toy, go back to empty mode
          Some(Mode::Empty)
        }
      }
    };

    if let Some(new_mode) = new_mode {
      mode = new_mode;
    }

    ignite.surface().swap_buffers();

    store.sync(&mut ignite);
  }
}
