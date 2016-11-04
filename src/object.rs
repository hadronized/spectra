use nalgebra::ToHomogeneous;

use id::Id;
use model::Model;
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
