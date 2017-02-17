use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use time::precise_time_ns;

use bootstrap::{Context, Keyboard, Mouse, MouseMove, Scroll, Window};
pub use bootstrap::{Action, Key, MouseButton};
use camera::{Camera, Freefly};
use transform::Translation;

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
  start_time: u64,
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
      start_time: precise_time_ns(),
      kbd: kbd,
      mouse: mouse,
      cursor: cursor,
      scroll: scroll,
      window: window
    }
  }

  pub fn time(&self) -> f32 {
    (precise_time_ns() - self.start_time) as f32 * 1e-9
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

  pub fn step<R>(&mut self, fps: Option<u32>, mut draw_frame: R) -> bool where R: FnMut(f32) {
    let loop_start_time = precise_time_ns();

    if self.window.should_close() {
      return false;
    }

    let t = self.time();

    draw_frame(t);

    self.window.swap_buffers();

    // wait for next frame according to the wished FPS
    if let Some(fps) = fps {
      let fps = fps as f32;
      let elapsed_time_ms = ((precise_time_ns() - loop_start_time) as f64 * 1e-6) as i64;
      let sleep_time_ms = (1. / fps * 1e3) as i64 - elapsed_time_ms;

      if sleep_time_ms > 0 {
        thread::sleep(Duration::from_millis(sleep_time_ms as u64));
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
