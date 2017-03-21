use luminance::{Depth32F, Dim2, Flat, Framebuffer, Mode, RGBA32F, Tess, Texture, Uniform, Unit};
use luminance::pipeline::{Pipe, Pipeline, RenderCommand, ShadingCommand};
use luminance::tess::TessRender;
use std::ops::{Add, Mul, Sub};

pub use luminance::{Equation, Factor};

use color::RGBA;
use resource::{Res, ResCache};
use shader::Program;

/// Simple texture that can be embedded into a compositing graph.
pub type TextureLayer<'a> = &'a ColorMap;

pub type ColorMap = Texture<Flat, Dim2, RGBA32F>;
pub type DepthMap = Texture<Flat, Dim2, Depth32F>;

/// Render layer used to host renders.
pub struct RenderLayer<'a> {
  render: Box<Fn(&Framebuffer<Flat, Dim2, ColorMap, DepthMap>) + 'a>
}

impl<'a> RenderLayer<'a> {
  pub fn new<F>(render: F) -> Self where F: 'a + Fn(&Framebuffer<Flat, Dim2, ColorMap, DepthMap>) {
    RenderLayer {
      render: Box::new(render)
    }
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
  /// A single color.
  ///
  /// Keep in mind that such a node is great when you want to display a fullscreen colored quad but
  /// you shouldn’t use it for blending purpose. Adding color masking to your post-process is a
  /// better alternative and will avoid fillrate alteration.
  Color(RGBA),
  /// Composite node.
  ///
  /// Composite nodes are used to blend two compositing nodes according to a given `Equation` and
  /// two blending `Factor`s for source and destination, respectively.
  Composite(Box<Node<'a>>, Box<Node<'a>>, RGBA, Equation, Factor, Factor)
}

impl<'a> Node<'a> {
  /// Compose this node with another one.
  pub fn compose_with(self, rhs: Self, clear_color: RGBA, eq: Equation, src_fct: Factor, dst_fct: Factor) -> Self {
    Node::Composite(Box::new(self), Box::new(rhs), clear_color, eq, src_fct, dst_fct)
  }

  /// Compose this node over the other. In effect, the resulting node will replace any pixels covered
  /// by the right node by the ones of the left node unless the alpha value is different than `1`.
  /// In that case, an additive blending based on the alpha value of the left node will be performed.
  ///
  /// If you set the alpha value to `0` at a pixel in the left node, then the resulting pixel will be
  /// the one from the right node.
  pub fn over(self, rhs: Self) -> Self {
    rhs.compose_with(self, RGBA::new(0., 0., 0., 0.), Equation::Additive, Factor::SrcAlpha, Factor::SrcAlphaComplement)
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

impl<'a> From<RGBA> for Node<'a> {
  fn from(color: RGBA) -> Self {
    Node::Color(color)
  }
}

impl<'a> Add for Node<'a> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    self.compose_with(rhs, RGBA::new(0., 0., 0., 0.), Equation::Additive, Factor::One, Factor::One)
  }
}

impl<'a> Sub for Node<'a> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    self.compose_with(rhs, RGBA::new(0., 0., 0., 0.), Equation::Subtract, Factor::One, Factor::One)
  }
}

impl<'a> Mul for Node<'a> {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    self.compose_with(rhs, RGBA::new(1., 1., 1., 1.), Equation::Additive, Factor::Zero, Factor::SrcColor)
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
  pub fn new(w: u32, h: u32, cache: &mut ResCache) -> Self {
    Compositor {
      w: w,
      h: h,
      framebuffers: Vec::new(),
      free_framebuffers: Vec::new(),
      program: cache.get("spectra/compositing/forward.glsl", vec![FORWARD_SOURCE.sem("source")]).unwrap(),
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
    let fb_index = self.treat_node(root);

    {
      let fb = &self.framebuffers[fb_index];

      let screen = Framebuffer::default((self.w, self.h));

      let black = [0., 0., 0., 1.];
      let program = self.program.borrow();
      let uniforms_tex = [FORWARD_SOURCE.alter(Unit::new(0))];
      let tess_render = TessRender::from(&self.quad);
      let tess = &[Pipe::empty().uniforms(&uniforms_tex).unwrap(tess_render.clone())];
      let render_cmd = &[Pipe::new(RenderCommand::new(None, false, tess))];
      let shd_cmd = &[Pipe::new(ShadingCommand::new(&program, render_cmd))];
      let texture_set = &[&*fb.color_slot];

      Pipeline::new(&screen, black, texture_set, &[], shd_cmd).run();
    }

    self.dispose_framebuffer(fb_index);
  }

  /// Treat a node hierarchy and return the index  of the framebuffer that contains the result.
  fn treat_node(&mut self, node: Node) -> usize {
    match node {
      Node::Render(layer) => self.render(layer),
      Node::Texture(texture) => self.texturize(texture),
      Node::Color(color) => self.colorize(color),
      Node::Composite(left, right, clear_color, eq, src_fct, dst_fct) => self.composite(*left, *right, clear_color, eq, src_fct, dst_fct),
    }
  }

  fn render(&mut self, layer: RenderLayer) -> usize {
    let fb_index = self.pull_framebuffer();
    let fb = &self.framebuffers[fb_index];

    (layer.render)(&fb);

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

  fn colorize(&mut self, color: RGBA) -> usize {
    let fb_index = self.pull_framebuffer();
    let fb = &self.framebuffers[fb_index];

    let color = *color.as_ref();

    Pipeline::new(fb, color, &[], &[], &[]).run();

    fb_index
  }

  fn composite(&mut self, left: Node, right: Node, clear_color: RGBA, eq: Equation, src_fct: Factor, dst_fct: Factor) -> usize {
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
      let program = self.program.borrow();
      let uniforms_left = [FORWARD_SOURCE.alter(Unit::new(0))];
      let uniforms_right = [FORWARD_SOURCE.alter(Unit::new(1))];
      let tess_render = TessRender::from(&self.quad);
      let tess = &[
        Pipe::empty().uniforms(&uniforms_left).unwrap(tess_render.clone()),
        Pipe::empty().uniforms(&uniforms_right).unwrap(tess_render)
      ];
      let render_cmd = &[Pipe::new(RenderCommand::new((eq, src_fct, dst_fct), false, tess))];
      let shd_cmd = &[Pipe::new(ShadingCommand::new(&program, render_cmd))];
      let texture_set = &[
        &*left_fb.color_slot,
        &*right_fb.color_slot
      ];

      Pipeline::new(fb, *clear_color.as_ref(), texture_set, &[], shd_cmd).run();
    }

    // dispose both left and right framebuffers
    self.dispose_framebuffer(left_index);
    self.dispose_framebuffer(right_index);

    fb_index
  }
}
