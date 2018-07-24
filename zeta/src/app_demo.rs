use spectra::render::framebuffer::Framebuffer2D;
use spectra::sys::ignite::{Action, GraphicsContext, Ignite, Key, Surface, WindowEvent, WindowOpt};
use spectra::sys::time::Time;

/// Start a demo application.
///
/// A demo application automatically closes if the window is closed or the escape key is entered.
pub fn run<D>(demo: D) where D: Demo {
  let mut ignite = ignite!(800, 600, WindowOpt::default()).expect("ignite creation");
  let mut back_buffer = Framebuffer2D::back_buffer(ignite.surface().size());

  'app: loop {
    for event in ignite.surface().poll_events() {
      match event {
        WindowEvent::Key(Key::Escape, _, Action::Release, _) | WindowEvent::Close => break 'app,

        WindowEvent::FramebufferSize(w, h) => {
          back_buffer = Framebuffer2D::back_buffer([w as u32, h as u32]);
        }

        _ => ()
      }
    }

    ignite.fps_restricted(|ignite| {
      ignite.surface().swap_buffers();
    });
  }
}

pub trait Demo {
  fn render_frame(&mut self, t: Time);
}
