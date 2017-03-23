use color::RGB;
use linear::{Direction, Position};
use std::default::Default;

pub struct LightProp {
  pub diff: RGB,
  _pad_0: f32,
  pub spec: RGB,
  _pad_1: f32,
  pub gloss: f32,
  _pad_2: [f32; 3]
}

impl LightProp {
  pub fn new(diff: RGB, spec: RGB, gloss: f32) -> Self {
    LightProp {
      diff: diff,
      _pad_0: 0.,
      spec: spec,
      _pad_1: 0.,
      gloss: gloss,
      _pad_2: [0., 0., 0.]
    }
  }
}

impl Default for LightProp {
  fn default() -> Self {
    LightProp::new(
      RGB::new(0.6, 0.6, 0.7),
      RGB::new(0.6, 0.6, 0.7),
      10.
    )
  }
}

pub struct Light<L> {
  pub prop: LightProp,
  pub feature: L
}

impl<L> Light<L> {
  pub fn new(prop: LightProp, feature: L) -> Self {
    Light {
      prop: prop,
      feature: feature 
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightDir {
  dir: Direction,
  _pad: f32
}

impl From<Direction> for LightDir {
  fn from(dir: Direction) -> Self {
    LightDir {
      dir: dir,
      _pad: 0.
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightPos {
  pos: Position,
  _pad: f32
}

impl From<Position> for LightPos {
  fn from(pos: Position) -> Self {
    LightPos {
      pos: pos,
      _pad: 0.
    }
  }
}

pub type Dir = Light<LightDir>;
pub type Omni = Light<LightPos>;

