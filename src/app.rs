use std::thread;
use std::time::Duration;
use time::precise_time_ns;

use bootstrap::{Action, Context, Key, Keyboard, Mouse, MouseButton, MouseMove, Scroll, Window};
use camera::Freefly;
use entity::{Entity, Translation};

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
  /// Last known cursor position.
  last_cursor: [f64; 2],
  /// Is the mouse’s left button pressed down?
  left_down: bool,
  /// Is the mouse’s right button pressed down?
  right_down: bool,
}

impl App {
  pub fn init(kbd: Keyboard, mouse: Mouse, cursor: MouseMove, scroll: Scroll, window: Window) -> Self {
    App {
      start_time: precise_time_ns(),
      kbd: kbd,
      mouse: mouse,
      cursor: cursor,
      scroll: scroll,
      window: window,
      last_cursor: [0., 0.],
      left_down: false,
      right_down: false,
    }
  }

  pub fn time(&self) -> f32 {
    (precise_time_ns() - self.start_time) as f32 * 1e-9
  }

  pub fn handle_events<'a>(&mut self, freefly: &mut Entity<&'a Freefly>) -> bool {
    let mut freefly_tr = Translation::new(0., 0., 0.);
    let mut freefly_or = Translation::new(0., 0., 0.);

    while let Ok((key, action)) = self.kbd.try_recv() {
      if action == Action::Release {
        if key == Key::Escape {
          return false;
        }
      } else {
        match key {
          Key::W => freefly_tr += Translation::new(0., 0., 1.),
          Key::S => freefly_tr += Translation::new(0., 0., -1.),
          Key::A => freefly_tr += Translation::new(1., 0., 0.),
          Key::D => freefly_tr += Translation::new(-1., 0., 0.),
          Key::Q => freefly_tr += Translation::new(0., -1., 0.),
          Key::E => freefly_tr += Translation::new(0., 1., 0.),
          _ => {}
        }
      }
    }

    freefly.mv(freefly_tr);

    while let Ok((button, action)) = self.mouse.try_recv() {
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
        _ => {}
      }
    }

    while let Ok(xy) = self.cursor.try_recv() {
      if self.left_down {
        let (dx, dy) = (xy[0] - self.last_cursor[0], xy[1] - self.last_cursor[1]);
        freefly_or = Translation::new(dy as f32, dx as f32, 0.);
      } else if self.right_down {
        let (dx, _) = (xy[0] - self.last_cursor[0], xy[1] - self.last_cursor[1]);
        freefly_or = Translation::new(0., 0., dx as f32);
      }

      self.last_cursor = xy;
    }

    freefly.look_around(freefly_or);

    true
  }

  pub fn step<R>(&mut self, mut draw_frame: R) -> bool where R: FnMut(f32) {
    let loop_start_time = precise_time_ns();

    if self.window.should_close() {
      return false;
    }

    let t = self.time();

    draw_frame(t);

    self.window.swap_buffers();

    // wait for next frame according to the wished FPS
    let fps = 60.;
    let elapsed_time_ms = ((precise_time_ns() - loop_start_time) as f64 * 1e-6) as i64;
    let sleep_time_ms = (1. / fps * 1e3) as i64 - elapsed_time_ms;

    if sleep_time_ms > 0 {
      thread::sleep(Duration::from_millis(sleep_time_ms as u64));
    }

    true
  }
}
