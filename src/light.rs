use color::RGB;
use linear::{Direction, Position};
use std::default::Default;

pub struct LightProp {
  pub diff: RGB,
  pub spec: RGB,
  pub gloss: f32
}

pub struct Light<L> {
  pub prop: LightProp,
  pub feature: L
}

pub type Dir = Light<Direction>;
pub type Omni = Light<Position>;

impl LightProp {
  pub fn new(diff: RGB, spec: RGB, gloss: f32) -> Self {
    LightProp {
      diff: diff,
      spec: spec,
      gloss: gloss
    }
  }
}

impl Default for LightProp {
  fn default() -> Self {
    LightProp {
      diff: RGB::new(0.6, 0.6, 0.7),
      spec: RGB::new(0.6, 0.6, 0.7),
      gloss: 10.
    }
  }
}

impl<L> Light<L> {
  pub fn new(prop: LightProp, feature: L) -> Self {
    Light {
      prop: prop,
      feature: feature 
    }
  }
}
