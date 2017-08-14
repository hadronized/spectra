use luminance::blending::{Equation, Factor};
use luminance::pipeline::{BoundTexture, Gpu, pipeline};
use luminance::shader::program;
use luminance::tess::{Mode, Tess, TessRender, TessVertices};
use luminance::texture::{Dim2, Flat};
use luminance::vertex::{Vertex, VertexFormat};
use std::cell::RefCell;

use framebuffer::Framebuffer2D;
use resource::{Res, ResCache};
use shader::{Program, Uniform, UniformBuilder, UniformInterface, UniformWarning, UnwrapOrUnbound};
use text::TextTexture;
use texture::{Depth32F, R32F, RGBA32F, Texture};

/// Vertex used in overlay’s objects. Position coordinates are in *window space*.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vert {
  /// Position.
  pub pos: [f32; 3],
  /// Color.
  pub color: [f32; 4]
}

impl Vert {
  pub fn new(pos: [f32; 3], color: [f32; 4]) -> Self {
    Vert {
      pos: pos,
      color: color
    }
  }

  /// Convert the vertex’ position to *clip space*.
  ///
  /// > Note: the depth coordinate is not affected.
  fn to_clip_space(&self, converter: &UnitConverter) -> Self {
    let converted = converter.from_win_coord(self.pos[0], self.pos[1]);
    let pos_ = [converted[0], converted[1], self.pos[2]];

    Vert {
      pos: pos_,
      color: self.color
    }
  }
}

impl Vertex for Vert {
  fn vertex_format() -> VertexFormat {
    <([f32; 3], [f32; 4]) as Vertex>::vertex_format()
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle(pub Vert, pub Vert, pub Vert);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quad(pub Vert, pub Vert, pub Vert, pub Vert);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Disc {
  pub center: Vert,
  pub radius: f32
}

impl Disc {
  pub fn new(center: Vert, radius: f32) -> Self {
    Disc {
      center: center,
      radius: radius
    }
  }

  fn to_clip_space(&self, converter: &UnitConverter) -> Self {
    Disc {
      center: self.center.to_clip_space(converter),
      radius: self.radius * converter.rw
    }
  }
}

impl Vertex for Disc {
  fn vertex_format() -> VertexFormat {
    <(Vert, f32) as Vertex>::vertex_format()
  }
}

pub type Texture2D = Texture<Flat, Dim2, RGBA32F>;

#[derive(Copy, Clone)]
pub struct Text<'a> {
  text_texture: &'a TextTexture,
  left_lower: Vert
}

impl<'a> Text<'a> {
  pub fn new(text_texture: &'a TextTexture, left_lower: Vert) -> Self {
    Text {
      text_texture: text_texture,
      left_lower: left_lower
    }
  }
}

struct DiscUniforms {
  screen_ratio: Uniform<f32>
}

impl UniformInterface for DiscUniforms {
  fn uniform_interface(builder: UniformBuilder) -> program::Result<(Self, Vec<UniformWarning>)> {
    let mut warnings = Vec::new();

    let iface = DiscUniforms {
      screen_ratio: builder.ask("ratio").unwrap_or_unbound(&builder, &mut warnings)
    };

    Ok((iface, warnings))
  }
}

struct TextUniforms {
  sampler: Uniform<BoundTexture<Texture<Flat, Dim2, R32F>>>,
  pos: Uniform<[f32; 3]>,
  size: Uniform<[f32; 2]>,
  scale: Uniform<f32>,
  color: Uniform<[f32; 4]>
}

impl UniformInterface for TextUniforms {
  fn uniform_interface(builder: UniformBuilder) -> program::Result<(Self, Vec<UniformWarning>)> {
    let mut warnings = Vec::new();

    let iface = TextUniforms {
      sampler: builder.ask("text_texture").unwrap_or_unbound(&builder, &mut warnings),
      pos: builder.ask("pos").unwrap_or_unbound(&builder, &mut warnings),
      size: builder.ask("size").unwrap_or_unbound(&builder, &mut warnings),
      scale: builder.ask("scale").unwrap_or_unbound(&builder, &mut warnings),
      color: builder.ask("color").unwrap_or_unbound(&builder, &mut warnings)
    };

    Ok((iface, warnings))
  }
}

pub struct Overlay {
  ratio: f32,
  tri_program: Res<Program<Vert, (), ()>>,
  tris: RefCell<Tess<Vert>>,
  disc_program: Res<Program<Disc, (), DiscUniforms>>,
  discs: RefCell<Tess<Disc>>,
  text_program: Res<Program<[f32; 2], (), TextUniforms>>,
  text_quad: Tess<[f32; 2]>,
  unit_converter: UnitConverter,
}

impl Overlay {
  pub fn new(w: u32, h: u32, max_tris: usize, max_quads: usize, max_discs: usize, cache: &mut ResCache) -> Self {
    let tri_program = cache.get("spectra/overlay/triangle.glsl", ()).unwrap();
    let tris = Tess::new(Mode::Triangle, TessVertices::Reserve::<Vert>(max_tris * 3 + max_quads * 4), None);

    let disc_program = cache.get("spectra/overlay/disc.glsl", ()).unwrap();
    let discs = Tess::new(Mode::Point, TessVertices::Reserve::<Disc>(max_discs), None);

    let text_program = cache.get("spectra/overlay/text.glsl", ()).unwrap();

    let text_quad = Tess::attributeless(Mode::TriangleStrip, 4);

    Overlay {
      ratio: w as f32 / h as f32,
      tri_program: tri_program,
      tris: RefCell::new(tris),
      disc_program: disc_program,
      discs: RefCell::new(discs),
      text_program: text_program,
      text_quad: text_quad,
      unit_converter: UnitConverter::new(w, h)
    }
  }

  /// Dispatch render input primitives into GPU buffers.
  fn dispatch(&self, input: &RenderInput) -> (usize, usize) {
    let mut tris_ref = self.tris.borrow_mut();
    let mut tri_i = 0;
    if let Ok(mut tris) = tris_ref.as_slice_mut() {
      for &Triangle(a, b, c) in input.triangles {
        let abc = [a, b, c];

        for &v in &abc {
          tris[tri_i] = v.to_clip_space(&self.unit_converter);
          tri_i += 1;
        }
      }

      for &Quad(a, b, c, d) in input.quads {
        let abcd = [a, b, c, c, b, d];

        for &v in &abcd {
          tris[tri_i] = v.to_clip_space(&self.unit_converter);
          tri_i += 1;
        }
      }
    }

    let mut discs_ref = self.discs.borrow_mut();
    let mut disc_i = 0;
    if let Ok(mut discs) = discs_ref.as_slice_mut() {
      for disc in input.discs {
        discs[disc_i] = disc.to_clip_space(&self.unit_converter);
        disc_i += 1;
      }
    }

    (tri_i, disc_i)
  }

  pub fn render(&self, gpu: &Gpu, framebuffer: &Framebuffer2D<Texture<Flat, Dim2, RGBA32F>, Texture<Flat, Dim2, Depth32F>>, input: &RenderInput) {
    let (tri_vert_nb, disc_vert_nb) = self.dispatch(input);
 
    let tris_ref = self.tris.borrow();
    let tris = TessRender::one_sub(&tris_ref, tri_vert_nb);

    let discs_ref = self.discs.borrow();
    let discs = TessRender::one_sub(&discs_ref, disc_vert_nb);

    let text_quad = TessRender::one_whole(&self.text_quad);

    let tri_program = self.tri_program.borrow();
    let disc_program = self.disc_program.borrow();
    let text_program = self.text_program.borrow();

    pipeline(framebuffer, [0., 0., 0., 0.], |shd_gate| {
      shd_gate.shade(&tri_program, |rdr_gate, _| {
        rdr_gate.render(None, true, |tess_gate| {
          tess_gate.render(tris);
        });
      });

      shd_gate.shade(&disc_program, |rdr_gate, uniforms| {
        uniforms.screen_ratio.update(self.ratio);

        rdr_gate.render(None, true, |tess_gate| {
          tess_gate.render(discs);
        });
      });

      if let Some((texts, text_scale)) = input.texts {
        shd_gate.shade(&text_program, |rdr_gate, uniforms| {
          for text in texts {
            let blending = (Equation::Additive, Factor::One, Factor::SrcAlphaComplement);
            let [tex_w, tex_h] = text.text_texture.size();

            let texture = gpu.bind_texture(&**text.text_texture);

            uniforms.sampler.update(texture);
            uniforms.pos.update(text.left_lower.to_clip_space(&self.unit_converter).pos);
            uniforms.size.update(self.unit_converter.from_win_dim(tex_w as f32, tex_h as f32));
            uniforms.scale.update(text_scale);
            uniforms.color.update(text.left_lower.color);

            rdr_gate.render(blending, true, |tess_gate| {
              tess_gate.render(text_quad.clone());
            });
          }
        });
      }
    });
  }
}

#[derive(Clone)]
pub struct RenderInput<'a> {
  triangles: &'a [Triangle],
  quads: &'a [Quad],
  discs: &'a [Disc],
  texts: Option<(&'a [Text<'a>], f32)>
}

impl<'a> RenderInput<'a> {
  pub fn new() -> Self {
    RenderInput {
      triangles: &[],
      quads: &[],
      discs: &[],
      texts: None,
    }
  }

  pub fn triangles(self, triangles: &'a [Triangle]) -> Self {
    RenderInput {
      triangles: triangles,
      ..self
    }
  }

  pub fn quads(self, quads: &'a [Quad]) -> Self {
    RenderInput {
      quads: quads,
      ..self
    }
  }

  pub fn discs(self, discs: &'a [Disc]) -> Self {
    RenderInput {
      discs: discs,
      ..self
    }
  }

  pub fn texts(self, texts: &'a [Text<'a>], scale: f32) -> Self {
    RenderInput {
      texts: Some((texts, scale.max(0.))),
      ..self
    }
  }
}

/// A *unit converter* is used to position stuff in overlay in a universal and screen independent
/// way.
///
/// The idea is that you provide should handle *window space* coordinates only. This type will then
/// help compute normalized coordinates the renderer can understand.
///
/// For the record, *window space* coordinates are defined as below – given a viewport size of
/// *w × h*:
///
/// ^ y = h
/// |
/// |
/// |    x = w
/// O------>
///
/// The formula used to convert from *window space* to what we call *clip space* is:
///
/// > x' = 2x / w - 1
/// > y' = 2y / h - 1
#[derive(Copy, Clone, Debug, PartialEq)]
struct UnitConverter {
  rw: f32,
  rh: f32,
  twice_rw: f32,
  twice_rh: f32
}

impl UnitConverter {
  /// Build a unit converter by giving it the size of the viewport.
  pub fn new(w: u32, h: u32) -> Self {
    let rw = 1. / (w as f32);
    let rh = 1. / (h as f32);

    UnitConverter {
      rw: rw,
      rh: rh,
      twice_rw: 2. * rw,
      twice_rh: 2. * rh
    }
  }

  /// Convert from *window space* coordinates to clip space* ones.
  pub fn from_win_coord(&self, x: f32, y: f32) -> [f32; 2] {
    let x_ = x * self.twice_rw - 1.;
    let y_ = y * self.twice_rh - 1.;

    [x_.min(1.).max(-1.), y_.min(1.).max(-1.)]
  }

  /// Convert from *window space* dimensions to clip space* ones.
  pub fn from_win_dim(&self, w: f32, h: f32) -> [f32; 2] {
    let w_ = w * self.rw;
    let h_ = h * self.rh;

    [w_.min(1.).max(-1.), h_.min(1.).max(-1.)]
  }
}
