use luminance::{Depth32F, Dim2, Flat, Framebuffer, Mode, RGBA32F, Tess, Texture, Uniform, Unit};
use std::ops::{Add, Sub, Mul};

pub use luminance::{Equation, Factor};

use resource::Res;
use scene::Scene;
use shader::Program;

/// Layer render that can be embedded into a compositing graph.
pub type LayerRender<'a> = (&'a ColorMap, &'a DepthMap);
/// Simple texture that can be embedded into a compositing graph.
pub type LayerTexture<'a> = &'a ColorMap;

pub type ColorMap = Texture<Flat, Dim2, RGBA32F>;
pub type DepthMap = Texture<Flat, Dim2, Depth32F>;

/// Compositing node.
pub enum Node<'a> {
  /// A render node.
  ///
  /// Contains a color map and a depth map.
  Render(LayerRender<'a>),
  /// A texture node.
  ///
  /// Contains a single texture.
  Texture(LayerTexture<'a>),
  /// Composite node.
  ///
  /// Composite nodes are used to blend two compositing nodes according to a given `Equation` and
  /// two blending `Factor`s for source and destination, respectively.
  Composite(Box<Node<'a>>, Box<Node<'a>>, Equation, Factor, Factor)
}

impl<'a> Node<'a> {
  /// Compose this node with another one.
  pub fn compose_with(self, other: Self, eq: Equation, src_fct: Factor, dst_fct: Factor) -> Self {
    Node::Composite(Box::new(self), Box::new(other), eq, src_fct, dst_fct)
  }
}

impl<'a> From<LayerRender<'a>> for Node<'a> {
  fn from(layer: LayerRender<'a>) -> Self {
    Node::Render(layer)
  }
}

impl<'a> From<LayerTexture<'a>> for Node<'a> {
  fn from(texture: LayerTexture<'a>) -> Self {
    Node::Texture(texture)
  }
}

impl<'a> Add for Node<'a> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    self.compose_with(rhs, Equation::Additive, Factor::One, Factor::One)
  }
}

impl<'a> Sub for Node<'a> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    self.compose_with(rhs, Equation::Subtract, Factor::One, Factor::One)
  }
}

impl<'a> Mul for Node<'a> {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    self.compose_with(rhs, Equation::Additive, Factor::Zero, Factor::SrcColor)
  }
}

/// Compositor object; used to consume `Node`s and output to screen.
pub struct Compositor {
  // allocated framebuffers that might contain nodes’ output
  framebuffers: Vec<Framebuffer<Flat, Dim2, ColorMap, DepthMap>>,
  // free list of available framebuffers
  free_framebuffers: Vec<usize>,
  // program used to render nodes
  program: Res<Program>,
  // fullscreen quad for compositing
  quad: Tess
}

const FORWARD_SOURCE: &'static Uniform<Unit> = &Uniform::new(0);

impl Compositor {
  pub fn new(scene: &mut Scene) -> Self {
    Compositor {
      framebuffers: Vec::new(),
      free_framebuffers: Vec::new(),
      program: scene.get("spectra/compositing/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap(),
      quad: Tess::attributeless(Mode::TriangleStrip, 4)
    }
  }
}
