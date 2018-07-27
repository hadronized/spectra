//! Helpers to create applications with spectra.

pub use render::pipeline::Builder;
pub use sys::res::Store;
pub use sys::time::Time;
pub use sys::ignite::WindowOpt;

use render::framebuffer::Framebuffer2D;
use sys::res::{StoreError, StoreOpt};
use sys::ignite::{Action, Ignite, GraphicsContext, Key, Surface, WindowEvent};
use sys::time::Monotonic;

/// Class of demo applications.
///
/// A demo is basically just a single function that takes the current time and display something. If
/// you hit escape or close the window, the application must quit.
pub trait Demo: Sized {
  /// Context used to initialize the demo.
  type Context;
  /// Initialization error that might occur.
  type Error: Sized + From<DemoError>;

  /// Initialize the demo with a given store.
  fn init(store: &mut Store<Self::Context>, ctx: &mut Self::Context) -> Result<Self, Self::Error>;

  /// Render the demo at a given time. 
  fn render_at(&mut self, t: Time, back_buffer: &BackBuffer, builder: Builder);

  /// Resize the demo when the framebuffer gets resized.
  fn resize(&mut self, width: u32, height: u32);
}

#[derive(Debug)]
pub enum DemoError {
  CannotCreateStore(StoreError),
}

pub type BackBuffer = Framebuffer2D<(), ()>;

// Run a demo.
pub fn run_demo<T>(mut ignite: Ignite, ctx: &mut T::Context) -> Result<(), T::Error> where T: Demo {
  let store_opt = StoreOpt::default().set_root("data");
  let mut store: Store<T::Context> = Store::new(store_opt).map_err(DemoError::CannotCreateStore)?;

  // initialize the demo
  let mut demo = T::init(&mut store, ctx)?;

  // create a bunch of objects needed for rendering
  let mut back_buffer = Framebuffer2D::back_buffer(ignite.surface().size());

  // loop over time and run the demo
  let t_start = Monotonic::now();
  'run: loop {
    // treat events first
    for event in ignite.surface().poll_events() {
      match event {
        // quit event
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'run Ok(());
        }

        // resize event
        WindowEvent::FramebufferSize(w, h) => {
          let size = [w as u32, h as u32];

          back_buffer = Framebuffer2D::back_buffer(size);
          demo.resize(size[0], size[1]);
        }

        _ => ()
      }
    }

    // render a frame
    ignite.fps_restricted(|ignite| {
      let t = t_start.elapsed_secs();
      let surface = ignite.surface();
      let builder = surface.pipeline_builder();

      demo.render_at(t, &back_buffer, builder);
      surface.swap_buffers();
    });
  }
}
