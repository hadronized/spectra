//! A freefly camera with support directional sensitivities.

use cgmath::{ElementWise, InnerSpace, Rotation};

use linear::{Quat, V3};
use render::projection::{Perspective, Projectable, Projection};
use scene::camera::base::Camera;
use sys::event::{Action, Key, MouseButton};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Freefly {
  // sensitivities
  #[serde(default = "def_yaw_sens")]
  pub yaw_sens: f32,
  #[serde(default = "def_pitch_sens")]
  pub pitch_sens: f32,
  #[serde(default = "def_roll_sens")]
  pub roll_sens: f32,
  #[serde(default = "def_forward_sens")]
  pub forward_sens: f32,
  #[serde(default = "def_strafe_sens")]
  pub strafe_sens: f32,
  #[serde(default = "def_upward_sens")]
  pub upward_sens: f32,
  // projection
  pub perspective: Perspective
}

impl Freefly {
  pub fn new() -> Self {
    Freefly {
      yaw_sens: def_yaw_sens(),
      pitch_sens: def_pitch_sens(),
      roll_sens: def_roll_sens(),
      forward_sens: def_forward_sens(),
      strafe_sens: def_strafe_sens(),
      upward_sens: def_upward_sens(),
      perspective: Perspective::default(),
    }
  }
}

impl Default for Freefly {
  fn default() -> Self {
    Self::new()
  }
}

impl Projectable for Freefly {
  fn projection(&self) -> Projection {
    self.perspective.projection()
  }
}

impl Camera<Freefly> {
  pub fn mv(&mut self, dir: V3<f32>) {
    let p = &self.properties;
    let axis = dir.normalize().mul_element_wise(V3::new(p.strafe_sens, p.upward_sens, p.forward_sens)); // FIXME: so ugly…
    let v = self.orientation.invert().rotate_vector(axis);

    self.position -= v;
  }

  pub fn look_around(&mut self, dir: V3<f32>) {
    let p = &self.properties;

    fn orient(phi: f32, axis: V3<f32>) -> Quat<f32> {
      Quat::from_sv(phi, axis)
    }

    self.orientation = orient(p.yaw_sens * dir.y, V3::unit_y()) * self.orientation;
    self.orientation = orient(p.pitch_sens * dir.x, V3::unit_x()) * self.orientation;
    self.orientation = orient(p.roll_sens * dir.z, V3::unit_z()) * self.orientation;
  }
}

fn def_yaw_sens() -> f32 { 0.01 }
fn def_pitch_sens() -> f32 { 0.01 }
fn def_roll_sens() -> f32 { 0.01 }
fn def_forward_sens() -> f32 { 0.1 }
fn def_strafe_sens() -> f32 { 0.1 }
fn def_upward_sens() -> f32 { 0.1 }

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
  pub fn on_key(&self, cam: &mut Camera<Freefly>, key: &Key) {
    match *key {
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

  pub fn on_mouse_button(&mut self, button: &MouseButton, action: &Action) {
    match (*button, *action) {
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

