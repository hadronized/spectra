use luminance::{Depth32F, Dim2, Flat, Framebuffer, Mode, RGBA32F, RenderCommand, Tess, Texture,
                Uniform, Unit};
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
  /// A render node.
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
  // width
  w: u32,
  // height
  h: u32,
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
  pub fn new(w: u32, h: u32, scene: &mut Scene) -> Self {
    Compositor {
      w: w,
      h: h,
      framebuffers: Vec::new(),
      free_framebuffers: Vec::new(),
      program: scene.get("spectra/compositing/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap(),
      quad: Tess::attributeless(Mode::TriangleStrip, 4)
    }
  }

  /// Whenever a node must be composed, we need a framebuffer to render into. This function pulls a
  /// framebuffer to use (via self.framebuffers) by returing an index. It might allocate a new
  /// framebuffer if there isn’t enough framebuffers to be pulled.
  fn pull_framebuffer(&mut self) -> usize {
    self.free_framebuffers.pop().unwrap_or_else(|| {
      let framebuffer_index = self.framebuffers.len();

      let framebuffer = Framebuffer::new((self.w, self.h), 0).unwrap();
      self.framebuffers.push(framebuffer);

      framebuffer_index
    })
  }

  /// Whenever a node has finished being composed, we *might* need to release the framebuffer it has
  /// pulled. This funciton does that job.
  ///
  /// It never deallocated memory. It has an important property: once a framebuffer is pulled,
  /// calling that function will make it available for other nodes, improving memory usage for the
  /// next calls.
  fn release_framebuffer(&mut self, framebuffer_index: usize) {
    self.free_framebuffers.push(framebuffer_index);
  }

  /// Consume and display a compositing graph represented by its nodes.
  pub fn display(&mut self, root: Node) {
    self.consume_tagged_nodes(TaggedNode { node: root, framebuffer_index: None });
  }

  /// Consume a tagged node by pulling / releasing framebuffers on the fly and tagging its child as
  /// we go deeper.
  fn consume_tagged_nodes(&mut self, tagged_node: TaggedNode) {
    match tagged_node.node {
      Node::Render(render) => {

    }
  }
}

/// A tagged node.
///
/// A tagged node is a `Node` tagged with a *framebuffer index*, which can be present or not. If
/// it’s not present, that means that the node’s composition hasn’t occurred yet.
struct TaggedNode<'a> {
  node: Node<'a>,
  framebuffer_index: Option<usize>
}
