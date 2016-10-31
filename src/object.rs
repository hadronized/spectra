use nalgebra::ToHomogeneous;

use id::Id;
use model::Model;
use transform::{M44, Orientation, Position, Scale, Transformable, translation_matrix};

pub struct Object<'a> {
  model: Id<'a, Model>,
  position: Position,
  orientation: Orientation,
  scale: Scale
}

impl<'a> Transformable for Object<'a> {
  fn transform(&self) -> M44 {
    let m = translation_matrix(-self.position) * self.scale.to_mat() * self.orientation.to_rotation_matrix().to_homogeneous();
    m.as_ref().clone()
  }
}
