use nalgebra::{Quaternion, ToHomogeneous, Unit};
use serde_json::from_reader;
use std::path::Path;
use std::fs::File;

use id::Id;
use model::Model;
use resource::{Cache, Get, Load, LoadError};
use transform::{M44, Orientation, Position, Scale, Transformable, translation_matrix};

pub struct Object<'a> {
  pub model: Id<'a, Model>,
  pub position: Position,
  pub orientation: Orientation,
  pub scale: Scale
}

impl<'a> Transformable for Object<'a> {
  fn transform(&self) -> M44 {
    let m = translation_matrix(-self.position) * self.scale.to_mat() * self.orientation.to_rotation_matrix().to_homogeneous();
    m.as_ref().clone()
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectManifest {
  model: String,
  position: [f32; 3],
  orientation: [f32; 4],
  scale: [f32; 3]
}

impl<'a> Load<'a> for Object<'a> {
  type Args = ();

  fn load<P>(path: P, cache: &mut Cache<'a>, _: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    info!("loading object {:?}", path);

    // read the manifest
    let manifest: ObjectManifest = {
      let file = File::open("data/scene/objects/{}.json").map_err(|e| LoadError::FileNotFound(path.to_path_buf(), format!("{:?}", e)))?;
      from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?
    };

    // get the model id
    let model_id = cache.get_id(&manifest.model, ()).ok_or(LoadError::ConversionFailed(format!("unable to find model {} for object at {:?}", manifest.model, path)))?;

    Ok(Object {
      model: model_id,
      position: (&manifest.position).into(),
      orientation: Unit::new(&Quaternion::from(&manifest.orientation)),
      scale: (&manifest.scale).into()
    })
  }
}
