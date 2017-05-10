use clap::{App, Arg};
use luminance_glfw::{self, open_window};
pub use luminance_glfw::{Action, DeviceError, Key, MouseButton, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use camera::{Camera, Freefly};
use linear::V3;

type Time = f64;

/// Signals events can pass up back to their handlers to notify them how they have processed an
/// event. They’re three kinds of signals:
///
/// - `EventSig::Ignored`:  the event should be passed to other handlers the parents knows about –
///    if any – because it wasn’t handled (ignored);
///
/// - `EventSig::Handled`: the event has been correctly handled;
///
/// - `EventSig::Focused`: the event has been correctly handled, and the parent should consider that
///    the this handler has now an exclusive focus on that event;
///
/// - `EventSig::Aborted`: the event has been correctly handled and the parent handler should be
///    aborted. This signal is typically used to kill all the handlers chain and thus quit the
///    application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventSig {
  Ignored,
  Handled,
  Focused,
  Aborted
}

/// Class of event handlers.
///
/// All functions return a special object of type `EventSig`. An event report gives information
/// about how a handler has handled the event. See the documentation of `EventSig` for further
/// information.
///
/// All functions’ implementations default to `EventSig::Ignored`.
pub trait EventHandler {
  /// Implement this function if you want to react to key strokes.
  fn on_key(&mut self, _: Key, _: Action) -> EventSig { EventSig::Ignored }
  /// Implement this function if you want to react to mouse button events.
  fn on_mouse_button(&mut self, _: MouseButton, _: Action) -> EventSig { EventSig::Ignored }
  /// Implement this function if you want to react to cursor moves.
  fn on_cursor_move(&mut self, _: [f32; 2]) -> EventSig { EventSig::Ignored }
  /// Implement this function if you want to react to scroll events.
  fn on_scroll(&mut self, _: [f32; 2]) -> EventSig { EventSig::Ignored }
}

/// Empty handler.
///
/// This handler will just let pass all events without doing anything. It’s only useful for debug
/// purposes when you don’t want to bother with interaction – it doesn’t even let you close the
/// application!
#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub struct Unhandled;

impl EventHandler for Unhandled {}

/// Device object.
///
/// Upon bootstrapping, this type is created to add interaction and context handling.
pub struct Device {
  raw: luminance_glfw::Device,
  /// Some kind of epoch start the application started at.
  start_time: Instant
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
             -> Result<Self, DeviceError> {
    let options = App::new(title)
      .version(version)
      .author(author)
      .arg(Arg::with_name("width")
           .short("w")
           .long("width")
           .value_name("WIDTH")
           .help("Sets the width of the viewport used for render")
           .takes_value(true))
      .arg(Arg::with_name("height")
           .short("h")
           .long("height")
           .value_name("HEIGHT")
           .help("Sets the height of the viewport used for render")
           .takes_value(true))
      .arg(Arg::with_name("fullscreen")
           .short("f")
           .long("fullscreen")
           .value_name("FULLSCREEN")
           .help("Sets the viewport to be displayed in fullscreen mode")
           .takes_value(false))
      .get_matches();

    let width = options.value_of("width").map(|s| s.parse().unwrap_or(def_width)).unwrap_or(def_width);
    let height = options.value_of("height").map(|s| s.parse().unwrap_or(def_height)).unwrap_or(def_height);
    let fullscreen = options.is_present("fullscreen");

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

    let dev = open_window(win_dim, title, win_opt)?;

    info!("bootstrapping finished");

    Ok(Device {
      raw: dev,
      start_time: Instant::now()
    })
  }

  /// Width of the attached window.
  #[inline]
  pub fn width(&self) -> u32 {
    self.raw.width()
  }

  /// Height of the attached window.
  #[inline]
  pub fn height(&self) -> u32 {
    self.raw.height()
  }

  /// Current time, starting from the beginning of the creation of that object.
  pub fn time(&self) -> f64 {
    let elapsed = Instant::now() - self.start_time;
    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
  }

  /// Dispatch events to a handler.
  pub fn dispatch_events<H>(&self, handler: &mut H) -> bool where H: EventHandler {
    while let Ok((key, action)) = self.raw.kbd.try_recv() {
      if handler.on_key(key, action) == EventSig::Aborted {
        return false;
      }
    }

    while let Ok((button, action)) = self.raw.mouse.try_recv() {
      if handler.on_mouse_button(button, action) == EventSig::Aborted {
        return false;
      }
    }

    while let Ok(xy) = self.raw.cursor.try_recv() {
      if handler.on_cursor_move(xy) == EventSig::Aborted {
        return false;
      }
    }

    while let Ok(xy) = self.raw.scroll.try_recv() {
      if handler.on_scroll(xy) == EventSig::Aborted {
        return false;
      }
    }

    true
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
  pub fn step<FPS, R>(&mut self, fps: FPS, mut draw_frame: R) -> bool where FPS: Into<Option<u32>>, R: FnMut(Time) {
    let t = self.time();

    self.raw.draw(|| {
      draw_frame(t);
    });

    // wait for next frame according to the wished FPS
    if let Some(fps) = fps.into() {
      let fps = fps as f64;
      let max_time = 1. / fps;
      let elapsed_time = self.time() - t;

      if elapsed_time < max_time {
        let sleep_time = max_time - elapsed_time;
        thread::sleep(Duration::from_millis((sleep_time * 1e3) as u64));
      }
    }

    true
  }
}

#[macro_export]
macro_rules! bootstrap {
  ($def_width:expr, $def_height:expr, $win_opt:expr) => {{
    $crate::bootstrap::Device::new($def_width,
                                   $def_height,
                                   crate_version!(),
                                   crate_authors!(),
                                   crate_name!(),
                                   $win_opt)
  }}
}

/// Freefly handler.
///
/// This handler is very neat as it provides freefly interaction.
pub struct FreeflyHandler {
  camera: Rc<RefCell<Camera<Freefly>>>,
  left_down: bool,
  right_down: bool,
  last_cursor: [f32; 2]
}

impl FreeflyHandler {
  pub fn new(camera: Rc<RefCell<Camera<Freefly>>>) -> Self {
    FreeflyHandler {
      camera: camera,
      left_down: false,
      right_down: false,
      last_cursor: [0., 0.]
    }
  }

  fn move_camera_on_event(&mut self, key: Key) {
    let camera = &mut self.camera.borrow_mut();

    match key {
      Key::W => camera.mv(V3::new(0., 0., 1.)),
      Key::S => camera.mv(V3::new(0., 0., -1.)),
      Key::A => camera.mv(V3::new(1., 0., 0.)),
      Key::D => camera.mv(V3::new(-1., 0., 0.)),
      Key::Q => camera.mv(V3::new(0., -1., 0.)),
      Key::E => camera.mv(V3::new(0., 1., 0.)),
      _ => ()
    }
  }

  fn orient_camera_on_event(&mut self, cursor: [f32; 2]) {
    let camera = &mut self.camera.borrow_mut();

    if self.left_down {
      let (dx, dy) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(V3::new(dy, dx, 0.));
    } else if self.right_down {
      let (dx, _) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(V3::new(0., 0., dx));
    }

    self.last_cursor = cursor;
  }
}

impl EventHandler for FreeflyHandler {
  fn on_key(&mut self, key: Key, action: Action) -> EventSig {
    match action {
      Action::Press | Action::Repeat => self.move_camera_on_event(key),
      Action::Release => if key == Key::Escape { return EventSig::Aborted }
    }

    EventSig::Handled
  }

  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> EventSig {
    match (button, action) {
      (MouseButton::Button1, Action::Press) => {
        self.left_down = true;
      },
      (MouseButton::Button1, Action::Release) => {
        self.left_down = false;
      },
      (MouseButton::Button2, Action::Press) => {
        self.right_down = true;
      },
      (MouseButton::Button2, Action::Release) => {
        self.right_down = false;
      },
      _ => ()
    }

    EventSig::Handled
  }

  fn on_cursor_move(&mut self, dir: [f32; 2]) -> EventSig {
    self.orient_camera_on_event(dir);
    EventSig::Handled
  }
}
