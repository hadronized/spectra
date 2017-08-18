use serde_json::from_reader;
use std::path::{Path, PathBuf};
use std::fs::File;

use linear::{M44, Quat, V3};
use model::{ObjModel, ObjModelKey};
use resource::{CacheKey, Load, LoadError, LoadResult, Res, Store};
use scale::Scale;
use transform::{Transform, Transformable};

pub type ObjObject = Object<ObjModel>;

#[derive(Clone)]
pub struct Object<M> {
  pub model: Res<M>,
  pub position: V3<f32>,
  pub orientation: Quat<f32>,
  pub scale: Scale
}

impl<M> Object<M> {
  pub fn new(model: Res<M>, position: V3<f32>, orientation: Quat<f32>, scale: Scale) -> Self {
    Object {
      model: model,
      position: position,
      orientation: orientation,
      scale: scale
    }
  }
}

impl<Vertex> Transformable for Object<Vertex> {
  fn transform(&self) -> Transform {
    (M44::from_translation(-self.position) * M44::from(self.scale) * M44::from(self.orientation)).into()
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectManifest {
  model: String,
  #[serde(default = "def_position")]
  position: [f32; 3],
  #[serde(default = "def_orientation")]
  orientation: [f32; 4],
  #[serde(default = "def_scale")]
  scale: [f32; 3]
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObjObjectKey(pub String);

impl CacheKey for ObjObjectKey {
  type Target = ObjObject;
}

impl Load for ObjObject {
  type Key = ObjObjectKey;

  fn key_to_path(key: &Self::Key) -> PathBuf {
    key.0.clone().into()
  }

  fn load<P>(path: P, cache: &mut Store) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    // read the manifest
    let manifest: ObjectManifest = {
      let file = File::open(path).map_err(|_| LoadError::FileNotFound(path.to_path_buf()))?;
      from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?
    };

    let model = cache.get(&ObjModelKey::new(&manifest.model)).ok_or(LoadError::ConversionFailed(format!("unable to find model {} for object at {:?}", manifest.model, path)))?;

    Ok((Object {
      model: model,
      position: manifest.position.into(),
      orientation: manifest.orientation.into(),
      scale: manifest.scale.into()
    }).into())
  }
}

fn def_position() -> [f32; 3] { [0., 0., 0.] }
fn def_orientation() -> [f32; 4] { [1., 0., 0., 0.] }
fn def_scale() -> [f32; 3] { [1., 1., 1.] }
