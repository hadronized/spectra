use clap::{App, Arg};
pub use luminance::context::GraphicsContext;
pub use luminance_windowing::{Surface, WindowDim, WindowOpt};
pub use luminance_glfw::surface::GlfwSurface;
pub use luminance_glfw::event::{Action, Key, MouseButton, WindowEvent};
pub use luminance_glfw::error::GlfwSurfaceError;
use std::thread;
use std::time::{Duration, Instant};

pub type Time = f64;

pub type Fps = f64;

pub struct Ignite {
  /// Glfw surface.
  surface: GlfwSurface,
  /// Some kind of epoch start the application started at.
  start_time: Instant,
  /// Frame rate limit to use when rendering. If none, no limit.
  framerate_limit_ms: Option<Fps>
}

impl Ignite {
  /// Ignite from CLI.
  pub fn new_from_cli(
    def_width: u32,
    def_height: u32,
    version: &str,
    author: &str,
    title: &str,
    win_opt: WindowOpt
  ) -> Result<Self, GlfwSurfaceError> {
    let options = App::new(title)
      .version(version)
      .author(author)
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
      .get_matches();

    let width = options.value_of("width").map(|s| s.parse().unwrap_or(def_width)).unwrap_or(def_width);
    let height = options.value_of("height").map(|s| s.parse().unwrap_or(def_height)).unwrap_or(def_height);
    let fullscreen = options.is_present("fullscreen");
    let framerate_limit_hz: Option<u16> = options.value_of("framerate-limit").and_then(|l| l.parse().ok());
    let framerate_limit_ms = framerate_limit_hz.map(|hz| 1. / (hz as f64));

    // build the WindowDim
    let win_dim = if fullscreen {
      if options.is_present("width") && options.is_present("height") {
        WindowDim::FullscreenRestricted(width, height)
      } else {
        WindowDim::Fullscreen
      }
    } else {
      WindowDim::Windowed(width, height)
    };

    info!("{} starting", title);
    info!("window mode: {:?}", win_dim);
    info!("window options: {:?}", win_opt);

    let surface = GlfwSurface::new(win_dim, title, win_opt)?;
    let ignite = Ignite {
      surface,
      start_time: Instant::now(),
      framerate_limit_ms
    };

    Ok(ignite)
  }

  /// Get access to the underlying `Surface`.
  pub fn surface(&mut self) -> &mut GlfwSurface {
    &mut self.surface
  }

  /// Current time, starting from the beginning of the creation of that object.
  pub fn time(&self) -> f64 {
    let elapsed = Instant::now() - self.start_time;
    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
  }

  pub fn fps_restricted<F>(&mut self, f: F) where F: FnOnce(&mut Self) {
    let start_t = self.time();

    f(self);

    // wait for next frame according to the wished FPS
    if let Some(framerate_limit_ms) = self.framerate_limit_ms {
      let elapsed_time = self.time() - start_t;

      if elapsed_time < framerate_limit_ms {
        let sleep_time = framerate_limit_ms - elapsed_time;
        thread::sleep(Duration::from_millis((sleep_time * 1e3) as u64));
      }
    }
  }
}

#[macro_export]
macro_rules! ignite {
  ($def_width:expr, $def_height:expr, $win_opt:expr) => {{
    $crate::sys::ignite::Ignite::new_from_cli($def_width,
                                              $def_height,
                                              crate_version!(),
                                              crate_authors!(),
                                              crate_name!(),
                                              $win_opt)
  }}
}
