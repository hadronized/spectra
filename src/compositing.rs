use luminance::framebuffer::Framebuffer;
use luminance::pixel::{Depth32F, RGBA32F};
use luminance::tess::{Mode, Tess};
use luminance::texture::{Dim2, Flat, Texture, Unit};
use luminance::pipeline::Pipeline;
use luminance::shader::program;

pub use luminance::blending::{Equation, Factor};

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
  forward_program: Res<Program<QuadVert, (), ComposeUniforms>>,
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

impl Compositor {
  /// Construct a new `Compositor` that will output to a screen which dimension is `w × h`.
  pub fn new(w: u32, h: u32, cache: &mut ResCache) -> Self {
    Compositor {
      w: w,
      h: h,
      framebuffers: Vec::new(),
      free_framebuffers: Vec::new(),
      forward_program: cache.get("spectra/compositing/forward.glsl", ()).unwrap(),
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
    let fb_index = self.pull_framebuffer();

    {
      let fb = &self.framebuffers[fb_index];

      for layer in layers {
        (layer.render)(fb);
      }

      let screen = Framebuffer::default([self.w, self.h]);
      Pipeline::new(&screen, [0., 0., 0., 1.], &[&*fb.color_slot()], &[]).enter(|shd_gate| {
        shd_gate.new(&self.forward_program.borrow(), &[], &[]).enter(|rdr_gate, uniforms| {
          uniforms.source.update(Unit::new(0));

          rdr_gate.new(None, false, &[], &[]).enter(|tess_gate| {
            let quad = &self.quad;
            tess_gate.render(quad.into(), &[], &[])
          });
        });
      });
    }

    self.dispose_framebuffer(fb_index);
  }
}
