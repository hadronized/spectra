use luminance::{Depth32F, Dim2, Flat, Framebuffer, Mode, RGBA32F, Tess, Texture, Uniform, Unit};
use luminance::pipeline::{Pipe, Pipeline, RenderCommand, ShadingCommand};
use luminance::tess::TessRender;
use std::ops::{Add, Sub, Mul};

pub use luminance::{Equation, Factor};

use resource::Res;
use scene::Scene;
use shader::Program;

/// Simple texture that can be embedded into a compositing graph.
pub type TextureLayer<'a> = &'a ColorMap;

pub type ColorMap = Texture<Flat, Dim2, RGBA32F>;
pub type DepthMap = Texture<Flat, Dim2, Depth32F>;

/// Render layer used to host renders.
pub struct RenderLayer<'a> {
  shading_commands: Vec<Pipe<'a, ShadingCommand<'a>>>
}

impl<'a> RenderLayer<'a> {
  pub fn new() -> Self {
    RenderLayer {
      shading_commands: Vec::new()
    }
  }

  pub fn push_shading_command(&mut self, shading_command: Pipe<'a, ShadingCommand<'a>>) {
    self.shading_commands.push(shading_command);
  }
}

/// Compositing node.
pub enum Node<'a> {
  /// A render node.
  ///
  /// Contains render layer.
  Render(RenderLayer<'a>),
  /// A texture node.
  ///
  /// Contains a single texture.
  Texture(TextureLayer<'a>),
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

impl<'a> From<RenderLayer<'a>> for Node<'a> {
  fn from(layer: RenderLayer<'a>) -> Self {
    Node::Render(layer)
  }
}

impl<'a> From<TextureLayer<'a>> for Node<'a> {
  fn from(texture: TextureLayer<'a>) -> Self {
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
  // program used to compose nodes
  program: Res<Program>,
  // attributeless fullscreen quad for compositing
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

  /// Whenever a node has finished being composed, we *might* need to dispose the framebuffer it has
  /// pulled. This funciton does that job.
  ///
  /// It never deallocates memory. It has an important property: once a framebuffer is pulled,
  /// calling that function will make it available for other nodes, improving memory usage for the
  /// next calls.
  #[inline]
  fn dispose_framebuffer(&mut self, framebuffer_index: usize) {
    self.free_framebuffers.push(framebuffer_index);
  }

  /// Consume and display a compositing graph represented by its nodes.
  pub fn display(&mut self, root: Node) {
    unimplemented!()
    //self.consume_tagged_nodes(TaggedNode { node: root, framebuffer_index: None });
  }

  /// Treat a node hierarchy and return the index  of the framebuffer that contains the result.
  fn treat_node(&mut self, node: Node) -> usize {
    match node {
      Node::Render(layer) => self.render(layer),
      Node::Texture(texture) => self.texturize(texture),
      Node::Composite(left, right, eq, src_fct, dst_fct) => self.composite(*left, *right, eq, src_fct, dst_fct),
    }
  }

  fn render(&mut self, layer: RenderLayer) -> usize {
    let fb_index = self.pull_framebuffer();
    let fb = &self.framebuffers[fb_index];
    let black = [0., 0., 0., 1.];

    Pipeline::new(fb, black, &[], &[], &layer.shading_commands).run();

    fb_index
  }

  fn texturize(&mut self, texture: TextureLayer) -> usize {
    let fb_index = self.pull_framebuffer();
    let fb = &self.framebuffers[fb_index];

    let black = [0., 0., 0., 1.];
    let program = self.program.borrow();
    let uniforms_tex = [FORWARD_SOURCE.alter(Unit::new(0))];
    let tess_render = TessRender::from(&self.quad);
    let tess = &[Pipe::empty().uniforms(&uniforms_tex).unwrap(tess_render.clone())];
    let render_cmd = &[Pipe::new(RenderCommand::new(None, false, tess))];
    let shd_cmd = &[Pipe::new(ShadingCommand::new(&program, render_cmd))];
    let texture_set = &[&**texture];

    Pipeline::new(fb, black, texture_set, &[], shd_cmd).run();

    fb_index
  }

  fn composite(&mut self, left: Node, right: Node, eq: Equation, src_fct: Factor, dst_fct: Factor) -> usize {
    let left_index = self.treat_node(left);
    let right_index = self.treat_node(right);

    assert!(left_index < self.framebuffers.len());
    assert!(right_index < self.framebuffers.len());

    let fb_index = self.pull_framebuffer();

    {
      let fb = &self.framebuffers[fb_index];

      let left_fb = &self.framebuffers[left_index];
      let right_fb = &self.framebuffers[right_index];

      // compose
      let black = [0., 0., 0., 1.];
      let program = self.program.borrow();
      let uniforms_left = [FORWARD_SOURCE.alter(Unit::new(0))];
      let uniforms_right = [FORWARD_SOURCE.alter(Unit::new(1))];
      let tess_render = TessRender::from(&self.quad);
      let tess = &[
        Pipe::empty().uniforms(&uniforms_left).unwrap(tess_render.clone()),
        Pipe::empty().uniforms(&uniforms_right).unwrap(tess_render)
      ];
      let render_cmd = &[Pipe::new(RenderCommand::new(Some((eq, src_fct, dst_fct)), false, tess))];
      let shd_cmd = &[Pipe::new(ShadingCommand::new(&program, render_cmd))];
      let texture_set = &[
        &*left_fb.color_slot,
        &*right_fb.color_slot
      ];

      Pipeline::new(fb, black, texture_set, &[], shd_cmd).run();
    }

    // dispose both left and right framebuffers
    self.dispose_framebuffer(left_index);
    self.dispose_framebuffer(right_index);

    fb_index
  }
}
