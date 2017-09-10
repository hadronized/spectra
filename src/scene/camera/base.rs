//! Base code of camera.

use serde::de::DeserializeOwned;
use serde_json::from_reader;
use std::default::Default;
use std::fmt;
use std::fs::File;
use std::hash;
use std::marker::PhantomData;
use std::path::PathBuf;

use linear::{M44, Quat, V3};
use render::projection::{Projectable, Projection};
use scene::transform::{Transform, Transformable};
use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};

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

#[derive(Deserialize)]
struct Manifest<P> {
  position: [f32; 3],
  orientation: [f32; 4],
  #[serde(default)]
  properties: P
}

#[derive(Eq, PartialEq)]
pub struct CameraKey<A> {
  pub key: String,
  _a: PhantomData<*const A>
}

impl<A> CameraKey<A> {
  pub fn new(key: &str) -> Self {
    CameraKey {
      key: key.to_owned(),
      _a: PhantomData
    }
  }
}

impl<A> Clone for CameraKey<A> {
  fn clone(&self) -> Self {
    CameraKey {
      key: self.key.clone(),
      ..*self
    }
  }
}

impl<A> fmt::Debug for CameraKey<A> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.key.fmt(f)
  }
}

impl<A> hash::Hash for CameraKey<A> {
  fn hash<H>(&self, hasher: &mut H) where H: hash::Hasher {
    self.key.hash(hasher)
  }
}

impl<A> CacheKey for CameraKey<A> where A: 'static {
  type Target = Camera<A>;
}

impl<A> StoreKey for CameraKey<A> where A: 'static {
  fn key_to_path(&self) -> PathBuf {
    self.key.clone().into()
  }
}

impl<A> Load for Camera<A> where A: 'static + Default + DeserializeOwned {
  fn load<K>(key: &K, _: &mut Store) -> Result<LoadResult<Self>, LoadError> where K: StoreKey<Target = Self> {
    let path = key.key_to_path();

    let manifest: Manifest<A> = {
      let file = File::open(&path).map_err(|_| LoadError::FileNotFound(path))?;
      from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?
    };

    Ok((Camera {
      position: manifest.position.into(),
      orientation: manifest.orientation.into(),
      properties: manifest.properties
    }).into())
  }
}

