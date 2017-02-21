use gl;
use glfw::{self, SwapInterval};
use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use time::{Duration, SteadyTime};

use camera::{Camera, Freefly};
use transform::Translation;

pub use glfw::{Action, Context, CursorMode, Key, MouseButton, Window};

#[derive(Clone, Copy, Debug)]
pub enum WindowDim {
  Windowed(u32, u32),
  FullScreen,
  FullScreenRestricted(u32, u32)
}

pub type Keyboard = mpsc::Receiver<(Key, Action)>;
pub type Mouse = mpsc::Receiver<(MouseButton, Action)>;
pub type MouseMove = mpsc::Receiver<[f64; 2]>;
pub type Scroll = mpsc::Receiver<[f64; 2]>;

pub fn bootstrap(dim: WindowDim, title: &'static str) -> (u32, u32, Keyboard, Mouse, MouseMove, Scroll, Window) {
  info!("{} starting", title);
  info!("window mode: {:?}", dim);

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  // OpenGL hints
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
  glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));

  // open a window in windowed or fullscreen mode
  let (mut window, events, w, h) = match dim {
    WindowDim::Windowed(w, h) => {
      let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::Windowed).expect("Failed to create GLFW window.");
      (window, events, w, h)
    },
    WindowDim::FullScreen => {
      glfw.with_primary_monitor(|glfw, monitor| {
        let monitor = monitor.unwrap();
        let vmode = monitor.get_video_mode().expect("primary monitor’s video mode");
        let (w, h) = (vmode.width, vmode.height);

        let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::FullScreen(monitor)).expect("Failed to create GLFW window.");
        (window, events, w, h)
      })
    },
    WindowDim::FullScreenRestricted(w, h) => {
      glfw.with_primary_monitor(|glfw, monitor| {
        let monitor = monitor.unwrap();

        let (window, events) = glfw.create_window(w, h, title, glfw::WindowMode::FullScreen(monitor)).expect("Failed to create GLFW window.");
        (window, events, w, h)
      })
    }
  };

  deb!("opened window");

  window.make_current();

  // FIXME: use a target instead
  if cfg!(feature = "release") {
    deb!("hiding cursor");
    window.set_cursor_mode(CursorMode::Disabled);
  }

  window.set_key_polling(true);
  window.set_cursor_pos_polling(true);
  window.set_mouse_button_polling(true);
  window.set_scroll_polling(true);
  glfw.set_swap_interval(SwapInterval::Sync(1));

  deb!("initializing OpenGL pointers");

  // init OpenGL
  gl::load_with(|s| window.get_proc_address(s) as *const c_void);

  // create channels to stream keyboard and mouse events
  let (kbd_snd, kbd_rcv) = mpsc::channel();
  let (mouse_snd, mouse_rcv) = mpsc::channel();
  let (mouse_move_snd, mouse_move_rcv) = mpsc::channel();
  let (scroll_snd, scroll_rcv) = mpsc::channel();

  deb!("spawning the event thread");
  let _ = thread::spawn(move || {
    loop {
      glfw.wait_events();

      for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::WindowEvent::Key(key, _, action, _) => {
              let _ = kbd_snd.send((key, action));
            },
            glfw::WindowEvent::MouseButton(button, action, _) => {
              let _ = mouse_snd.send((button, action));
            },
            glfw::WindowEvent::CursorPos(x, y) => {
              let _ = mouse_move_snd.send([x, y]);
            },
            glfw::WindowEvent::Scroll(x, y) => {
              let _ = scroll_snd.send([x, y]);
            },
            _ => {},
        }
      }
    }
  });

  deb!("bootstrapping finished");
  (w, h, kbd_rcv, mouse_rcv, mouse_move_rcv, scroll_rcv, window)
}

#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub struct Unhandled;

/// Class of keyboard handlers.
pub trait KeyboardHandler {
  fn on_key(&mut self, key: Key, action: Action) -> bool;
}

impl KeyboardHandler for Unhandled {
  fn on_key(&mut self, _: Key, _: Action) -> bool { true }
}

/// Class of mouse button handlers.
pub trait MouseButtonHandler {
  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> bool;
}

impl MouseButtonHandler for Unhandled {
  fn on_mouse_button(&mut self, _: MouseButton, _: Action) -> bool { true }
}

/// Class of mouse move handlers.
pub trait CursorHandler {
  fn on_cursor_move(&mut self, cursor_pos: [f64; 2]) -> bool;
}

impl CursorHandler for Unhandled {
  fn on_cursor_move(&mut self, _: [f64; 2]) -> bool { true }
}

/// Class of scroll handlers.
pub trait ScrollHandler {
  fn on_scroll(&mut self, scroll_dir: [f64; 2]) -> bool;
}

impl ScrollHandler for Unhandled {
  fn on_scroll(&mut self, _: [f64; 2]) -> bool { true }
}

/// All common stuff goes here.
pub struct App {
  /// Some kind of epoch start the application started at.
  start_time: SteadyTime,
  /// Keyboard receiver.
  kbd: Keyboard,
  /// Mouse receiver.
  mouse: Mouse,
  /// Cursor receiver.
  cursor: MouseMove,
  /// Scroll receiver.
  scroll: Scroll,
  /// Window.
  window: Window,
}

impl App {
  pub fn init(kbd: Keyboard, mouse: Mouse, cursor: MouseMove, scroll: Scroll, window: Window) -> Self {
    App {
      start_time: SteadyTime::now(),
      kbd: kbd,
      mouse: mouse,
      cursor: cursor,
      scroll: scroll,
      window: window
    }
  }

  pub fn time(&self) -> f64 {
    (SteadyTime::now() - self.start_time).num_nanoseconds().unwrap() as f64 * 1e-9
  }

  pub fn dispatch_events<H>(&self, handler: &mut H) -> bool
      where H: KeyboardHandler + MouseButtonHandler + CursorHandler + ScrollHandler {
    while let Ok((key, action)) = self.kbd.try_recv() {
      if !handler.on_key(key, action) {
        return false;
      }
    }

    while let Ok((button, action)) = self.mouse.try_recv() {
      if !handler.on_mouse_button(button, action) {
        return false;
      }
    }

    while let Ok(xy) = self.cursor.try_recv() {
      if !handler.on_cursor_move(xy) {
        return false;
      }
    }

    while let Ok(xy) = self.scroll.try_recv() {
      if !handler.on_scroll(xy) {
        return false;
      }
    }

    true
  }

  pub fn step<R>(&mut self, fps: Option<u32>, mut draw_frame: R) -> bool where R: FnMut(f64) {
    let loop_start_time = SteadyTime::now();

    if self.window.should_close() {
      return false;
    }

    let t = self.time();

    draw_frame(t);

    self.window.swap_buffers();

    // wait for next frame according to the wished FPS
    if let Some(fps) = fps {
      let fps = fps as f32;
      let elapsed_time = SteadyTime::now() - loop_start_time;
      let max_time = Duration::nanoseconds((1. / (fps as f64) * 1e9) as i64);

      if elapsed_time > max_time {
        let sleep_time = max_time - elapsed_time;
        thread::sleep(sleep_time.to_std().unwrap());
      }
    }

    true
  }
}

/// Debug handler.
pub struct DebugHandler {
  camera: Rc<RefCell<Camera<Freefly>>>,
  left_down: bool,
  right_down: bool,
  last_cursor: [f64; 2]
}

impl DebugHandler {
  pub fn new(camera: Rc<RefCell<Camera<Freefly>>>) -> Self {
    DebugHandler {
      camera: camera,
      left_down: false,
      right_down: false,
      last_cursor: [0., 0.]
    }
  }

  fn move_camera_on_event(&mut self, key: Key) {
    let camera = &mut self.camera.borrow_mut();

    match key {
      Key::W => camera.mv(Translation::new(0., 0., 1.)),
      Key::S => camera.mv(Translation::new(0., 0., -1.)),
      Key::A => camera.mv(Translation::new(1., 0., 0.)),
      Key::D => camera.mv(Translation::new(-1., 0., 0.)),
      Key::Q => camera.mv(Translation::new(0., -1., 0.)),
      Key::E => camera.mv(Translation::new(0., 1., 0.)),
      _ => ()
    }
  }

  fn orient_camera_on_event(&mut self, cursor: [f64; 2]) {
    let camera = &mut self.camera.borrow_mut();

    if self.left_down {
      let (dx, dy) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(Translation::new(dy as f32, dx as f32, 0.));
    } else if self.right_down {
      let (dx, _) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      camera.look_around(Translation::new(0., 0., dx as f32));
    }

    self.last_cursor = cursor;
  }
}

impl KeyboardHandler for DebugHandler {
  fn on_key(&mut self, key: Key, action: Action) -> bool {
    match action {
      Action::Press | Action::Repeat => self.move_camera_on_event(key),
      Action::Release => if key == Key::Escape { return false }
    }

    true
  }
}

impl MouseButtonHandler for DebugHandler {
  fn on_mouse_button(&mut self, button: MouseButton, action: Action) -> bool {
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

    true
  }
}

impl CursorHandler for DebugHandler {
  fn on_cursor_move(&mut self, dir: [f64; 2]) -> bool {
    self.orient_camera_on_event(dir);
    true
  }
}

impl ScrollHandler for DebugHandler {
  fn on_scroll(&mut self, _: [f64; 2]) -> bool { true }
}
