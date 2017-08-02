use cgmath::{ElementWise, InnerSpace, Rotation};
use serde::de::DeserializeOwned;
use serde_json::from_reader;
use std::default::Default;
use std::fs::File;
use std::path::Path;

use linear::{M44, Quat, V3};
use projection::{Perspective, Projectable, Projection};
use resource::{Load, LoadError, LoadResult, ResCache};
use transform::{Transform, Transformable};

#[derive(Clone, Debug)]
pub struct Camera<P> {
  pub position: V3<f32>,
  pub orientation: Quat<f32>,
  pub properties: P
}

impl<P> Camera<P> {
  pub fn new(position: V3<f32>, orientation: Quat<f32>, properties: P) -> Self {
    Camera {
      position: position,
      orientation: orientation,
      properties: properties
    }
  }
}

impl<P> Default for Camera<P> where P: Default {
  fn default() -> Self {
    Camera::new(V3::new(0., 0., 0.),
                Quat::from_sv(1., V3::new(0., 0., 0.)),
                P::default())
  }
}

impl<T> Projectable for Camera<T> where T: Projectable {
  fn projection(&self) -> Projection {
    self.properties.projection()
  }
}

impl<P> Transformable for Camera<P> {
  fn transform(&self) -> Transform {
    (M44::from(self.orientation) * M44::from_translation(-self.position)).into()
  }
}

#[derive(Deserialize)]
struct Manifest<P> {
  position: [f32; 3],
  orientation: [f32; 4],
  #[serde(default)]
  properties: P
}

impl<A> Load for Camera<A> where A: Default + DeserializeOwned {
  type Args = ();

  const TY_STR: &'static str = "cameras";

  fn load<P>(path: P, _: &mut ResCache, _: Self::Args) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    let manifest: Manifest<A> = {
      let file = File::open(path).map_err(|_| LoadError::FileNotFound(path.to_path_buf()))?;
      from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?
    };

    Ok((Camera {
      position: manifest.position.into(),
      orientation: manifest.orientation.into(),
      properties: manifest.properties
    }).into())
  }
}

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
