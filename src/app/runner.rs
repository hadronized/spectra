//! Demo runner.

use clap::{App, Arg};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance_glfw::surface::{
  Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt
};
use std::time::Instant;
use warmy::{Store, StoreOpt};

use crate::app::demo::Demo;
use crate::time::Monotonic;

/// Main runner. Release-oriented.
///
/// This runner shall be used whenever wanted to release a demo with the minimal features enabled,
/// that is, a running demo that one can close by hitting escape or closing the window.
pub struct ReleaseRunner {
  surface: GlfwSurface,
  /// Some kind of epoch start the application started at.
  start_time: Instant,
}

impl ReleaseRunner {
  fn create_clap_app<'a, 'b>(title: &str) -> App<'a, 'b> {
    App::new(title)
      .arg(Arg::with_name("width")
           .short("w")
           .long("width")
           .value_name("WIDTH")
           .help("Set the width of the viewport used for render")
           .takes_value(true))
      .arg(Arg::with_name("height")
           .short("h")
           .long("height")
           .value_name("HEIGHT")
           .help("Set the height of the viewport used for render")
           .takes_value(true))
      .arg(Arg::with_name("fullscreen")
           .short("f")
           .long("fullscreen")
           .value_name("FULLSCREEN")
           .help("Set the viewport to be displayed in fullscreen mode")
           .takes_value(false))
      .arg(Arg::with_name("framerate-limit")
           .short("r")
           .long("limit-framerate-to")
           .value_name("FRAMERATE_LIMIT")
           .help("Set the framerate limit")
           .takes_value(true))
  }

  pub fn run<D>(
    title: &str,
    def_width: u32,
    def_height: u32
  ) where D: Demo {
    // get CLI options
    let cli_options = Self::create_clap_app(title).get_matches();
    let width = cli_options.value_of("width").map(|s| s.parse().unwrap_or(def_width)).unwrap_or(def_width);
    let height = cli_options.value_of("height").map(|s| s.parse().unwrap_or(def_height)).unwrap_or(def_height);
    let fullscreen = cli_options.is_present("fullscreen");
    let framerate_limit_hz: Option<u16> = cli_options.value_of("framerate-limit").and_then(|l| l.parse().ok());
    let framerate_limit_ms = framerate_limit_hz.map(|hz| 1. / (hz as f64));

    // build the WindowDim
    let win_dim = if fullscreen {
      if cli_options.is_present("width") && cli_options.is_present("height") {
        WindowDim::FullscreenRestricted(width, height)
      } else {
        WindowDim::Fullscreen
      }
    } else {
      WindowDim::Windowed(width, height)
    };

    let win_opt = WindowOpt::default().hide_cursor(true);

    // create the rendering surface
    let surface = GlfwSurface::new(win_dim, title, win_opt).expect("GLFW surface");

    // create the store
    let store_opt = StoreOpt::default().set_root("data");
    let mut store: Store<D::Context> = Store::new(store_opt).expect("store creation");

    // initialize the demo
    let mut demo = D::init(&mut store).expect("demo initialization");

    // create a bunch of objects needed for rendering
    let mut back_buffer = Framebuffer::back_buffer(surface.size());

    // loop over time and run the demo
    let start_time = Monotonic::now();
    'run: loop {
      // treat events first
      for event in surface.poll_events() {
        match event {
          // quit event
          WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
            break 'run;
          }

          // resize event
          WindowEvent::FramebufferSize(w, h) => {
            let size = [w as u32, h as u32];

            back_buffer = Framebuffer::back_buffer(size);
            demo.resize(size[0], size[1]);
          }

          _ => ()
        }
      }

      // render a frame
      let t = start_time.elapsed_secs();
      let builder = surface.pipeline_builder();
      demo.render(t, &back_buffer, builder);

      surface.swap_buffers();
    }
  }
}
