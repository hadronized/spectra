//! Base code of camera.

use serde::de::DeserializeOwned;
use serde_json::from_reader;
use serde_json::error::Error as JsonError;
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use linear::{M44, Quat, V3};
use render::projection::{Projectable, Projection};
use scene::transform::{Transform, Transformable};
use sys::resource::{DebugRes, Load, Loaded, PathKey, Store, load_with};

#[derive(Clone, Debug)]
pub struct Camera<P> {
  pub position: V3<f32>,
  pub orientation: Quat<f32>,
  pub properties: P
}

impl<P> Camera<P> {
  pub fn new(position: V3<f32>, orientation: Quat<f32>, properties: P) -> Self {
    Camera {
      position,
      orientation,
      properties
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Manifest<P> {
  position: [f32; 3],
  orientation: [f32; 4],
  #[serde(default)]
  properties: P
}

impl<A> DebugRes for Camera<A> {
  const TYPE_DESC: &'static str = "camera";
}

impl<A> Load for Camera<A> where A: 'static + Default + DeserializeOwned {
  type Key = PathKey;
  type Error = CameraError;

  fn load(key: Self::Key, _: &mut Store) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_path();

    load_with::<Self, _, _>(path, move || {
      let manifest: Manifest<A> = {
        let file = File::open(path).map_err(|_| CameraError::FileNotFound(path.to_owned()))?;
        from_reader(file).map_err(CameraError::ParseFailed)?
      };

      Ok((Camera {
        position: manifest.position.into(),
        orientation: manifest.orientation.into(),
        properties: manifest.properties
      }).into())
    })
  }
  
  impl_reload_passthrough!();
}

#[derive(Debug)]
pub enum CameraError {
  FileNotFound(PathBuf),
  ParseFailed(JsonError)
}

impl fmt::Display for CameraError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for CameraError {
  fn description(&self) -> &str {
    match *self {
      CameraError::FileNotFound(_) => "file not found",
      CameraError::ParseFailed(_) => "parse failed"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      CameraError::ParseFailed(ref json_error) => Some(json_error),
      _ => None
    }
  }
}
