//! The debug runner.

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance_glfw::surface::{
  Action, GlfwSurface, Key as GlfwKey, Surface, WindowDim, WindowEvent, WindowOpt
};
use structopt::StructOpt;
use warmy::{Store, StoreOpt};

use crate::app::demo::Demo;
use crate::resource::context::Context;
use crate::resource::key::Key;
use crate::time::{DurationSpec, Monotonic};

/// Debug runner.
///
/// This runner shall be used whenever wanted to debug a demo.
pub struct Runner;

#[derive(StructOpt, Debug)]
struct Opt {
  /// Width of the viewport.
  #[structopt(short = "w", long = "width")]
  width: Option<u32>,

  /// Height of the viewport.
  #[structopt(short = "h", long = "height")]
  height: Option<u32>,

  /// Shall the viewport be in fullscreen mode?
  #[structopt(short = "f", long = "fullscreen")]
  fullscreen: bool,

  /// Set a maximum runtime duration. Whenever the time arrives at this duration limit, it will
  /// wrap around to 0. If unset, the demo will run with a forever increasing time.
  ///
  /// The syntax is “MmSs”, where M is optional. M must be a natural specifiying the number of
  /// minutes and S a natural specifying the number of seconds. 30s will then be 30 seconds and 1m42
  /// will be 1 minute and 42 seconds. The number of seconds must not exceed 59.
  #[structopt(short = "z", long = "wrap-at")]
  wrap_at: Option<DurationSpec>,

  /// Start the demo at a given time.
  #[structopt(short = "s", long = "start-at", default_value = "0s")]
  start_at: DurationSpec
}

impl Runner {
  pub fn run<D>(
    title: &str,
    def_width: u32,
    def_height: u32,
    mut context: D::Context
  ) where D: Demo {
    info!(context.logger(), "starting « {} »", title);

    // get CLI options
    let opt = Opt::from_args();
    let width = opt.width.unwrap_or(def_width);
    let height = opt.height.unwrap_or(def_height);
    let fullscreen = opt.fullscreen;

    // build the WindowDim
    let win_dim = if fullscreen {
      if opt.width.is_some() && opt.height.is_some() {
        info!(context.logger(), "window mode: fullscreen restricted ({}×{})", width, height);
        WindowDim::FullscreenRestricted(width, height)
      } else {
        info!(context.logger(), "window mode: fullscreen");
        WindowDim::Fullscreen
      }
    } else {
      info!(context.logger(), "window mode: windowed ({}×{})", width, height);
      WindowDim::Windowed(width, height)
    };

    let win_opt = WindowOpt::default();

    // create the rendering surface
    let mut surface = GlfwSurface::new(win_dim, title, win_opt).expect("GLFW surface");

    // create the store
    let store_opt = StoreOpt::default().set_root("data");
    let mut store: Store<D::Context, Key> = Store::new(store_opt).expect("store creation");

    // initialize the demo
    let mut demo = D::init(&mut store, &mut context).expect("demo initialization");

    // create a bunch of objects needed for rendering
    let mut back_buffer = Framebuffer::back_buffer(surface.size());

    // loop over time and run the demo
    let start_time = Monotonic::now();
    let start_at = opt.start_at;
    let wrap_at = opt.wrap_at;

    info!(context.logger(), "initialized; running…");

    'run: loop {
      // treat events first
      for event in surface.poll_events() {
        match event {
          // quit event
          WindowEvent::Close | WindowEvent::Key(GlfwKey::Escape, _, Action::Release, _) => {
            break 'run;
          }

          // resize event
          WindowEvent::FramebufferSize(w, h) => {
            let size = [w as u32, h as u32];

            back_buffer = Framebuffer::back_buffer(size);
            demo.resize(&mut context, size[0], size[1]);
          }

          _ => ()
        }
      }

      // render a frame
      let t = start_time.elapsed_secs();
      let t = if let Some(wrap_t) = wrap_at { t.wrap_around(wrap_t.into()) } else { t };
      let t = t.offset(start_at.into());
      let builder = surface.pipeline_builder();

      demo.render(&mut context, t, &back_buffer, builder);
      surface.swap_buffers();
    }
  }
}
