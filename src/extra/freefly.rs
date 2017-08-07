use std::default::Default;

use event::{Action, Key, MouseButton};
pub use camera::{Camera, Freefly};
use linear::V3;

pub struct FreeflyState {
  left_down: bool,
  right_down: bool,
  last_cursor: [f32; 2]
}

impl Default for FreeflyState {
  fn default() -> Self {
    FreeflyState {
      left_down: false,
      right_down: false,
      last_cursor: [0., 0.]
    }
  }
}

impl FreeflyState {
  pub fn on_key(&self, cam: &mut Camera<Freefly>, key: Key) {
    match key {
      Key::W => cam.mv(V3::new(0., 0., 1.)),
      Key::S => cam.mv(V3::new(0., 0., -1.)),
      Key::A => cam.mv(V3::new(1., 0., 0.)),
      Key::D => cam.mv(V3::new(-1., 0., 0.)),
      Key::Q => cam.mv(V3::new(0., -1., 0.)),
      Key::E => cam.mv(V3::new(0., 1., 0.)),
      _ => ()
    }
  }

  pub fn on_cursor_move(&mut self, cam: &mut Camera<Freefly>, cursor: [f32; 2]) {
    if self.left_down {
      let (dx, dy) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      cam.look_around(V3::new(dy, dx, 0.));
    } else if self.right_down {
      let (dx, _) = (cursor[0] - self.last_cursor[0], cursor[1] - self.last_cursor[1]);
      cam.look_around(V3::new(0., 0., dx));
    }

    self.last_cursor = cursor;
  }

  pub fn on_mouse_button(&mut self, button: MouseButton, action: Action) {
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
  }
}
