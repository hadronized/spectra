use luminance::framebuffer::Framebuffer;
use luminance::pixel::{Depth32F, RGBA32F};
use luminance::tess::{Mode, Tess};
use luminance::texture::{Dim2, Flat, Texture, Unit};
use luminance::pipeline::Pipeline;
use luminance::shader::program;
use luminance::tess::TessRender;
use std::ops::{Add, Mul, Sub};

pub use luminance::blending::{Equation, Factor};

use color::RGBA;
use framebuffer::Framebuffer2D;
use resource::{Res, ResCache};
use shader::{Program, Uniform, UniformBuilder, UniformInterface, UniformWarning, UnwrapOrUnbound};

pub type ColorMap = Texture<Flat, Dim2, RGBA32F>;
pub type DepthMap = Texture<Flat, Dim2, Depth32F>;

/// Render layer.
///
/// A render layer is used whenever a render must be completed. It can be for 3D purposes – i.e. a
/// scene render – or for 2D effects. Combinators are provided to help you build such a layer.
pub struct Layer<'a> {
  render: Box<Fn(&Framebuffer2D<ColorMap, DepthMap>) + 'a>
}

impl<'a> Layer<'a> {
  /// Lowest construct to build a `Layer`.
  ///
  /// This function takes a closure and turns it into a `Layer`, enabling you to borrow things on
  /// the fly inside the closure if needed.
  pub fn new<F>(f: F) -> Self where F: 'a + Fn(&Framebuffer2D<ColorMap, DepthMap>) {
    Layer {
      render: Box::new(f)
    }
  }
}

// /// Compositing node.
// pub enum Node<'a> {
//   /// A render node.
//   ///
//   /// A render node is used whenever a render must be completed. It can be for 3D purposes – i.e.
//   /// rendering a scene – or for 2D effects. Combinators are provided to help you build such a
//   /// `Node`.
//   Render(Rendered<'a>),
//   /// A node holding an `RGBA` color.
//   Color(RGBA),
//   /// Composite node.
//   ///
//   /// Composite nodes are used to blend two compositing nodes according to a given `Equation` and
//   /// two blending `Factor`s for source and destination, respectively.
//   Composite(Box<Node<'a>>, Box<Node<'a>>, RGBA, Equation, Factor, Factor),
// }
// 
// impl<'a> Node<'a> {
//   /// Compose this node with another one.
//   pub fn compose_with(self, rhs: Self, clear_color: RGBA, eq: Equation, src_fct: Factor, dst_fct: Factor) -> Self {
//     Node::Composite(Box::new(self), Box::new(rhs), clear_color, eq, src_fct, dst_fct)
//   }
// 
//   /// Compose this node over the other. In effect, the resulting node will replace any pixels covered
//   /// by the right node by the ones of the left node unless the alpha value is different than `1`.
//   /// In that case, an additive blending based on the alpha value of the left node will be performed.
//   ///
//   /// If you set the alpha value to `0` at a pixel in the left node, then the resulting pixel will be
//   /// the one from the right node.
//   pub fn over(self, rhs: Self) -> Self {
//     rhs.compose_with(self, RGBA::new(0., 0., 0., 0.), Equation::Additive, Factor::SrcAlpha, Factor::SrcAlphaComplement)
//   }
// }
// 
// impl<'a> From<RGBA> for Node<'a> {
//   fn from(color: RGBA) -> Self {
//     Node::Color(color)
//   }
// }
// 
// impl<'a> Add for Node<'a> {
//   type Output = Self;
// 
//   fn add(self, rhs: Self) -> Self {
//     self.compose_with(rhs, RGBA::new(0., 0., 0., 0.), Equation::Additive, Factor::One, Factor::One)
//   }
// }
// 
// impl<'a> Sub for Node<'a> {
//   type Output = Self;
// 
//   fn sub(self, rhs: Self) -> Self {
//     self.compose_with(rhs, RGBA::new(0., 0., 0., 0.), Equation::Subtract, Factor::One, Factor::One)
//   }
// }
// 
// impl<'a> Mul for Node<'a> {
//   type Output = Self;
// 
//   fn mul(self, rhs: Self) -> Self {
//     self.compose_with(rhs, RGBA::new(1., 1., 1., 1.), Equation::Additive, Factor::Zero, Factor::SrcColor)
//   }
// }
// 
/// Compositor object; used to consume `Layer`s and output to screen.
pub struct Compositor {
  // width
  w: u32,
  // height
  h: u32,
  // allocated framebuffers that might contain nodes’ output
  framebuffers: Vec<Framebuffer2D<ColorMap, DepthMap>>,
  // free list of available framebuffers
  free_framebuffers: Vec<usize>,
  // program used to compose nodes
  compose_program: Res<Program<QuadVert, (), ComposeUniforms>>,
  // program used to render scaled textures
  texture_program: Res<Program<QuadVert, (), TextureUniforms>>,
  // attributeless fullscreen quad for compositing
  quad: Tess<QuadVert>
}

type QuadVert = [f32; 2];

struct ComposeUniforms {
  source: Uniform<Unit>
}

impl UniformInterface for ComposeUniforms {
  fn uniform_interface(builder: UniformBuilder) -> program::Result<(Self, Vec<UniformWarning>)> {
    let mut warnings = Vec::new();

    let iface = ComposeUniforms {
      source: builder.ask("source").unwrap_or_unbound(&builder, &mut warnings)
    };

    Ok((iface, warnings))
  }
}

struct TextureUniforms {
  source: Uniform<Unit>,
  scale: Uniform<[f32; 2]>
}

impl UniformInterface for TextureUniforms {
  fn uniform_interface(builder: UniformBuilder) -> program::Result<(Self, Vec<UniformWarning>)> {
    let mut warnings = Vec::new();

    let iface = TextureUniforms {
      source: builder.ask("source").unwrap_or_unbound(&builder, &mut warnings),
      scale: builder.ask("scale").unwrap_or_unbound(&builder, &mut warnings)
    };

    Ok((iface, warnings))
  }
}

impl Compositor {
  /// Construct a new `Compositor` that will output to a screen which dimension is `w × h`.
  pub fn new(w: u32, h: u32, cache: &mut ResCache) -> Self {
    Compositor {
      w: w,
      h: h,
      framebuffers: Vec::new(),
      free_framebuffers: Vec::new(),
      compose_program: cache.get("spectra/compositing/forward.glsl", ()).unwrap(),
      texture_program: cache.get("spectra/compositing/texture.glsl", ()).unwrap(),
      quad: Tess::attributeless(Mode::TriangleStrip, 4)
    }
  }

  /// Whenever a layer must be composed, we need a framebuffer to render into. This function pulls a
  /// framebuffer to use (via self.framebuffers) by returning an index. It might allocate a new
  /// framebuffer if there isn’t enough framebuffers to be pulled.
  fn pull_framebuffer(&mut self) -> usize {
    self.free_framebuffers.pop().unwrap_or_else(|| {
      let framebuffer_index = self.framebuffers.len();

      let framebuffer = Framebuffer::new([self.w, self.h], 0).unwrap();
      self.framebuffers.push(framebuffer);

      framebuffer_index
    })
  }

  /// Whenever a layer has finished being composed, we *might* need to dispose the framebuffer it
  /// has pulled. This function does that job.
  ///
  /// It *never* deallocates memory. It brings in an important property: once a framebuffer is
  /// pulled, calling that function will make it available for other layers, improving memory usage
  /// for the next calls.
  #[inline]
  fn dispose_framebuffer(&mut self, framebuffer_index: usize) {
    self.free_framebuffers.push(framebuffer_index);
  }

  /// Consume and display a list of render layers.
  pub fn display<'a, L>(&mut self, layers: L) where L: IntoIterator<Item = &'a Layer<'a>> {
    let screen = Framebuffer::default([self.w, self.h]);
    let fb_index = self.pull_framebuffer();
    let fb = &self.framebuffers[fb_index];

    for layer in layers {
      (layer.render)(fb);
    }

    Pipeline::new(&screen, [0., 0., 0., 1.], &[&*fb.color_slot], &[]).enter(|shd_gate| {
      shd_gate.new(&self.compose_program.borrow(), &[], &[]).enter(|rdr_gate, uniforms| {
        uniforms.source.update(Unit::new(0));

        rdr_gate.new(None, false, &[], &[]).enter(|tess_gate| {
          let quad = &self.quad;
          tess_gate.render(quad.into(), &[], &[])
        });
      });
    });
  }
//   /// Consume and display a compositing graph represented by its nodes.
//   pub fn display(&mut self, root: Node) {
//     let fb_index = self.treat_node(root);
// 
//     {
//       let fb = &self.framebuffers[fb_index];
//       let screen = Framebuffer::default((self.w, self.h));
//       let compose_program = self.compose_program.borrow();
//       let tess_render = TessRender::from(&self.quad);
// 
//       Pipeline::new(&screen, [0., 0., 0., 1.], &[&*fb.color_slot], &[]).enter(|shd_gate| {
//         shd_gate.new(&compose_program, &[], &[]).enter(|rdr_gate, uniforms| {
//           rdr_gate.new(None, false, &[], &[]).enter(|tess_gate| {
//             uniforms.source.update(Unit::new(0));
// 
//             tess_gate.render(tess_render, &[], &[])
//           });
//         });
//       });
//     }
// 
//     self.dispose_framebuffer(fb_index);
//   }
// 
//   /// Treat a node hierarchy and return the index  of the framebuffer that contains the result.
//   fn treat_node(&mut self, node: Node) -> usize {
//     match node {
//       Node::Render(layer) => self.render(layer),
//       Node::Color(color) => self.colorize(color),
//       Node::Composite(left, right, clear_color, eq, src_fct, dst_fct) => self.composite(*left, *right, clear_color, eq, src_fct, dst_fct),
//     }
//   }
// 
//   fn render(&mut self, rendered: Rendered) -> usize {
//     let fb_index = self.pull_framebuffer();
//     let fb = &self.framebuffers[fb_index];
// 
//     rendered(&fb);
// 
//     fb_index
//   }
// 
//   fn texturize(&mut self, texture: ColorMap, opt_scale: Option<[f32; 2]>) -> usize {
//     let fb_index = self.pull_framebuffer();
//     let fb = &self.framebuffers[fb_index];
// 
//     let texture_program = self.texture_program.borrow();
//     let tess_render = TessRender::from(&self.quad);
//     let scale = opt_scale.unwrap_or([1., 1.]);
// 
//     Pipeline::new(fb, [0., 0., 0., 1.], &[&**texture], &[]).enter(|shd_gate| {
//       shd_gate.new(&texture_program, &[], &[]).enter(|rdr_gate, uniforms| {
//         rdr_gate.new(None, false, &[], &[]).enter(|tess_gate| {
//           uniforms.source.update(Unit::new(0));
//           uniforms.scale.update(scale);
// 
//           tess_gate.render(tess_render, &[], &[]);
//         });
//       });
//     });
// 
//     fb_index
//   }
// 
//   fn colorize(&mut self, color: RGBA) -> usize {
//     let fb_index = self.pull_framebuffer();
//     let fb = &self.framebuffers[fb_index];
// 
//     let color = *color.as_ref();
// 
//     Pipeline::new(fb, color, &[], &[]).enter(|_| {});
// 
//     fb_index
//   }
// 
//   fn composite(&mut self, left: Node, right: Node, clear_color: RGBA, eq: Equation, src_fct: Factor, dst_fct: Factor) -> usize {
//     let left_index = self.treat_node(left);
//     let right_index = self.treat_node(right);
// 
//     assert!(left_index < self.framebuffers.len());
//     assert!(right_index < self.framebuffers.len());
// 
//     let fb_index = self.pull_framebuffer();
// 
//     {
//       let fb = &self.framebuffers[fb_index];
// 
//       let left_fb = &self.framebuffers[left_index];
//       let right_fb = &self.framebuffers[right_index];
// 
//       let texture_set = &[
//         &*left_fb.color_slot,
//         &*right_fb.color_slot
//       ];
//       let compose_program = self.compose_program.borrow();
//       let tess_render = TessRender::from(&self.quad);
// 
//       Pipeline::new(fb, *clear_color.as_ref(), texture_set, &[]).enter(|shd_gate| {
//         shd_gate.new(&compose_program, &[], &[], &[]).enter(|rdr_gate| {
//           rdr_gate.new((eq, src_fct, dst_fct), false, &[], &[], &[]).enter(|tess_gate| {
//             let uniforms = [FORWARD_SOURCE.alter(Unit::new(0))];
//             tess_gate.render(tess_render.clone(), &uniforms, &[], &[]);
// 
//             let uniforms = [FORWARD_SOURCE.alter(Unit::new(1))];
//             tess_gate.render(tess_render, &uniforms, &[], &[]);
//           });
//         });
//       });
//     }
// 
//     // dispose both left and right framebuffers
//     self.dispose_framebuffer(left_index);
//     self.dispose_framebuffer(right_index);
// 
//     fb_index
//   }
// 
//   fn fullscreen_effect(&mut self, program: &Program) -> usize {
//     let fb_index = self.pull_framebuffer();
//     let fb = &self.framebuffers[fb_index];
// 
//     let tess_render = TessRender::from(&self.quad);
// 
//     Pipeline::new(fb, [0., 0., 0., 1.], &[], &[]).enter(|shd_gate| {
//       shd_gate.new(&program, &[], &[], &[]).enter(|rdr_gate| {
//         rdr_gate.new(None, false, &[], &[], &[]).enter(|tess_gate| {
//           tess_gate.render(tess_render, &[], &[], &[]);
//         });
//       });
//     });
// 
//     fb_index
//   }
}
