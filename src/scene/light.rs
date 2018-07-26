use std::default::Default;

use linear::V3;
use render::color::RGB;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightProp {
  pub diff: RGB,
  _pad_0: f32,
  pub spec: RGB,
  pub gloss: f32,
}

impl LightProp {
  pub fn new(diff: RGB, spec: RGB, gloss: f32) -> Self {
    LightProp {
      diff: diff,
      _pad_0: 0.,
      spec: spec,
      gloss: gloss,
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
  dir: V3<f32>,
  _pad: f32
}

impl From<V3<f32>> for LightDir {
  fn from(dir: V3<f32>) -> Self {
    LightDir {
      dir: dir,
      _pad: 0.
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightPos {
  pos: V3<f32>,
  _pad: f32
}

impl From<V3<f32>> for LightPos {
  fn from(pos: V3<f32>) -> Self {
    LightPos {
      pos: pos,
      _pad: 0.
    }
  }
}

pub type Dir = Light<LightDir>;
pub type Omni = Light<LightPos>;
