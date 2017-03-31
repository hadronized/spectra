use luminance::blending::{Equation, Factor};
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::{Pipeline, RenderCommand, ShadingCommand};
use luminance::tess::{Mode, Tess, TessRender, TessVertices};
use luminance::texture::{Dim2, Flat, Unit};
use luminance::vertex::{Vertex, VertexFormat};
use std::cell::RefCell;

use compositing::{ColorMap, DepthMap};
use framebuffer::Framebuffer2D;
use resource::{Res, ResCache};
use shader::{Program, Uniform};
use text::TextTexture;
use texture::{RGBA32F, Texture};

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

const DISC_SCREEN_RATIO: &'static Uniform<f32> = &Uniform::new(0);

const TEXT_SAMPLER: &'static Uniform<Unit> = &Uniform::new(0);
const TEXT_POS: &'static Uniform<[f32; 3]> = &Uniform::new(1);
const TEXT_SIZE: &'static Uniform<[f32; 2]> = &Uniform::new(2);
const TEXT_SCALE: &'static Uniform<f32> = &Uniform::new(3);
const TEXT_COLOR: &'static Uniform<[f32; 4]> = &Uniform::new(4);

pub struct Overlay {
  ratio: f32,
  tri_program: Res<Program>,
  tris: RefCell<Tess>,
  disc_program: Res<Program>,
  discs: RefCell<Tess>,
  text_program: Res<Program>,
  text_quad: Tess,
  unit_converter: UnitConverter,
}

impl Overlay {
  pub fn new(w: u32, h: u32, max_tris: usize, max_quads: usize, max_discs: usize, cache: &mut ResCache) -> Self {
    let tri_program = cache.get("spectra/overlay/triangle.glsl", vec![]).unwrap();
    let tris = Tess::new(Mode::Triangle, TessVertices::Reserve::<Vert>(max_tris * 3 + max_quads * 4), None);

    let disc_program = cache.get("spectra/overlay/disc.glsl", vec![DISC_SCREEN_RATIO.sem("ratio")]).unwrap();
    let discs = Tess::new(Mode::Point, TessVertices::Reserve::<Disc>(max_discs), None);

    let text_program = cache.get("spectra/overlay/text.glsl", vec![
      TEXT_SAMPLER.sem("text_texture"),
      TEXT_POS.sem("pos"),
      TEXT_SIZE.sem("size"),
      TEXT_SCALE.sem("scale"),
      TEXT_COLOR.sem("color")
    ]).unwrap();

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
    let mut tris = tris_ref.as_slice_mut().unwrap();
    let mut tri_i = 0;

    let mut discs_ref = self.discs.borrow_mut();
    let mut discs = discs_ref.as_slice_mut().unwrap();
    let mut disc_i = 0;

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

    for disc in input.discs {
      discs[disc_i] = disc.to_clip_space(&self.unit_converter);
      disc_i += 1;
    }

    (tri_i, disc_i)
  }

  pub fn render(&self, framebuffer: &Framebuffer2D<ColorMap, DepthMap>, input: &RenderInput) {
    let (tri_vert_nb, disc_vert_nb) = self.dispatch(input);
 
    let tris_ref = self.tris.borrow();
    let tris = TessRender::one_sub(&tris_ref, tri_vert_nb);

    let discs_ref = self.discs.borrow();
    let discs = TessRender::one_sub(&discs_ref, disc_vert_nb);

    let text_quad = TessRender::one_whole(&self.text_quad);

    let tri_program = self.tri_program.borrow();
    let disc_program = self.disc_program.borrow();
    let text_program = self.text_program.borrow();

    Pipeline::new(framebuffer, [0., 0., 0., 0.], &[], &[]).enter(|shd_gate| {
      shd_gate.new(&tri_program, &[], &[], &[]).enter(|rdr_gate| {
        rdr_gate.new(None, true, &[], &[], &[]).enter(|tess_gate| {
          tess_gate.render(tris, &[], &[], &[]);
        });
      });

      shd_gate.new(&disc_program, &[DISC_SCREEN_RATIO.alter(self.ratio)], &[], &[]).enter(|rdr_gate| {
        rdr_gate.new(None, true, &[], &[], &[]).enter(|tess_gate| {
          tess_gate.render(discs, &[], &[], &[]);
        });
      });

      if let Some((texts, text_scale)) = input.texts {
        shd_gate.new(&text_program, &[], &[], &[]).enter(|rdr_gate| {
          for text in texts {
            let blending = (Equation::Additive, Factor::One, Factor::SrcAlphaComplement);
            let (tex_w, tex_h) = text.text_texture.size();
            let uniforms = [
              TEXT_SAMPLER.alter(Unit::new(0)),
              TEXT_POS.alter(text.left_lower.to_clip_space(&self.unit_converter).pos),
              TEXT_SIZE.alter(self.unit_converter.from_win_dim(tex_w as f32, tex_h as f32)),
              TEXT_SCALE.alter(text_scale),
              TEXT_COLOR.alter(text.left_lower.color),
            ];

            rdr_gate.new(blending, true, &uniforms, &[&***text.text_texture], &[]).enter(|tess_gate| {
              tess_gate.render(text_quad.clone(), &[], &[], &[]);
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
