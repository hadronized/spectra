use id::Id;
use model::Model;
use transform::{M44, Orientation, Position, Scale, Transformable};

pub struct Object<'a> {
  model: Id<'a, Model>,
  position: Position,
  orientation: Orientation,
  scale: Scale
}

impl<'a> Transformable for Object<'a> {
  fn transform(&self) -> M44 {
  }
}
