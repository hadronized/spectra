use serde_json::from_reader;
use std::path::Path;
use std::fs::File;

use linear::{Matrix4, Quaternion, ToHomogeneous, Unit};
use model::Model;
use resource::{Load, LoadError, Res, ResCache};
use transform::{Orientation, Position, Scale, Transformable, translation_matrix};

#[derive(Clone, Debug)]
pub struct Object {
  pub model: Res<Model>,
  pub position: Position,
  pub orientation: Orientation,
  pub scale: Scale
}

impl Object {
  pub fn new(model: Res<Model>, position: Position, orientation: Orientation, scale: Scale) -> Self {
    Object {
      model: model,
      position: position,
      orientation: orientation,
      scale: scale
    }
  }
}

impl Transformable for Object {
  fn transform(&self) -> Matrix4<f32> {
    translation_matrix(-self.position) * self.scale.to_mat() * self.orientation.to_rotation_matrix().to_homogeneous()
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

impl Load for Object {
  type Args = ();

  const TY_STR: &'static str = "objects";

  fn load<P>(path: P, cache: &mut ResCache, _: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    info!("loading object {:?}", path);

    // read the manifest
    let manifest: ObjectManifest = {
      let file = File::open(path).map_err(|e| LoadError::FileNotFound(path.to_path_buf(), format!("{:?}", e)))?;
      from_reader(file).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?
    };

    let model = cache.get(&manifest.model, ()).ok_or(LoadError::ConversionFailed(format!("unable to find model {} for object at {:?}", manifest.model, path)))?;

    Ok(Object {
      model: model,
      position: (&manifest.position).into(),
      orientation: Unit::new(&Quaternion::from(&manifest.orientation)),
      scale: (&manifest.scale).into()
    })
  }
}

fn def_position() -> [f32; 3] { [0., 0., 0.] }
fn def_orientation() -> [f32; 4] { [1., 0., 0., 0.] }
fn def_scale() -> [f32; 3] { [1., 1., 1.] }
