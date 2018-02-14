//! Bootstrap your application!
//!
//! This module is used to create an application with spectra. Its aim is to quickly setup a spectra
//! app so that you can start working on interesting things.
use clap::{App, Arg};
use luminance_glfw;
pub use luminance_glfw::{Device as DeviceTrait, GLFWDeviceError, Key, MouseButton, WindowDim, WindowOpt};
use std::thread;
use std::time::{Duration, Instant};

pub use sys::event::WindowEvent;

pub type Time = f64;

pub type Fps = f64;

/// Device object.
///
/// Upon bootstrapping, this type is created to add interaction and context handling.
pub struct Device {
  raw: luminance_glfw::GLFWDevice,
  /// Some kind of epoch start the application started at.
  start_time: Instant,
  /// Frame rate limit to use when rendering. If none, no limit.
  framerate_limit_ms: Option<Fps>
}

impl Device {
  /// Entry point.
  ///
  /// This function is the first one you have to call before anything else related to this crate.
  ///
  /// > Note: see `bootstrap!` macro for a better usage.
  pub fn new(def_width: u32,
             def_height: u32,
             version: &str,
             author: &str,
             title: &str,
             win_opt: WindowOpt)
             -> Result<Self, GLFWDeviceError> {
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

    let dev = luminance_glfw::GLFWDevice::new(win_dim, title, win_opt)?;

    info!("bootstrapping finished");

    Ok(Device {
      raw: dev,
      start_time: Instant::now(),
      framerate_limit_ms
    })
  }

  /// Size of the attached window.
  #[inline]
  pub fn size(&self) -> [u32; 2] {
    self.raw.size()
  }

  /// Current time, starting from the beginning of the creation of that object.
  pub fn time(&self) -> f64 {
    let elapsed = Instant::now() - self.start_time;
    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
  }

  /// Get the last event and pop it from the event queue. Supposed to be called in a loop.
  pub fn events<'a>(&'a mut self) -> impl Iterator<Item = WindowEvent> + 'a {
    self.raw.events()
  }

  /// Step function.
  ///
  /// This function provides two features:
  ///
  /// - it runs a *drawer* function, responsible of rendering a single frame, by passing it the
  ///   current time
  /// - it performs process idleing if you have requested a certain *framerate* – frame per second.
  ///
  /// The second feature is very neat because it lets you handle the scheduler off your application
  /// and then contribute to better CPU usage.
  ///
  /// > Note: if you pass `None`, no idleing will take place. However, you might be blocked by the
  /// *VSync* if enabled in your driver.
  pub fn step<R>(&mut self, mut draw_frame: R) -> bool where R: FnMut(Time) {
    let t = self.time();

    self.raw.draw(|| {
      draw_frame(t);
    });

    // wait for next frame according to the wished FPS
    if let Some(framerate_limit_ms) = self.framerate_limit_ms {
      let elapsed_time = self.time() - t;

      if elapsed_time < framerate_limit_ms {
        let sleep_time = framerate_limit_ms - elapsed_time;
        thread::sleep(Duration::from_millis((sleep_time * 1e3) as u64));
      }
    }

    true
  }
}

#[macro_export]
macro_rules! bootstrap {
  ($def_width:expr, $def_height:expr, $win_opt:expr) => {{
    $crate::sys::bootstrap::Device::new($def_width,
                                        $def_height,
                                        crate_version!(),
                                        crate_authors!(),
                                        crate_name!(),
                                        $win_opt)
  }}
}
