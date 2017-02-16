use luminance::{Dim2, Equation, Factor, Flat, Framebuffer, Mode, Pipe, Pipeline, RenderCommand, ShadingCommand, Tess, TessRender, TessVertices, Uniform, Unit, Vertex, VertexFormat};

use resource::Res;
use scene::Scene;
use shader::Program;
use text::TextTexture;
use texture::{RGBA32F, Texture};

/// Vertex used in overlay’s objects.
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
}

impl Vertex for Vert {
  fn vertex_format() -> VertexFormat {
    <([f32; 3], [f32; 4]) as Vertex>::vertex_format()
  }
}

pub struct Triangle(pub Vert, pub Vert, pub Vert);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Disc {
  center: Vert,
  radius: f32
}

impl Disc {
  pub fn new(center: Vert, radius: f32) -> Self {
    Disc {
      center: center,
      radius: radius
    }
  }
}

impl Vertex for Disc {
  fn vertex_format() -> VertexFormat {
    <(Vert, f32) as Vertex>::vertex_format()
  }
}

pub type Texture2D = Texture<Flat, Dim2, RGBA32F>;

pub struct Text<'a> {
  text_texture: &'a TextTexture,
  left_upper: Vert
}

impl<'a> Text<'a> {
  pub fn new(text_texture: &'a TextTexture, left_upper: Vert) -> Self {
    Text {
      text_texture: text_texture,
      left_upper: left_upper
    }
  }
}

const DISC_SCREEN_RATIO: &'static Uniform<f32> = &Uniform::new(0);

const TEXT_SAMPLER: &'static Uniform<Unit> = &Uniform::new(0);
const TEXT_POS: &'static Uniform<[f32; 3]> = &Uniform::new(1);
const TEXT_SIZE: &'static Uniform<[f32; 2]> = &Uniform::new(2);
const TEXT_SCALE: &'static Uniform<f32> = &Uniform::new(3);
const TEXT_COLOR: &'static Uniform<[f32; 4]> = &Uniform::new(4);

/// A renderer responsible of rendering shapes.
pub struct Renderer {
  framebuffer: Framebuffer<Flat, Dim2, Texture2D, ()>,
  ratio: f32,
  tri_program: Res<Program>,
  tris: Tess,
  tri_vert_nb: usize,
  disc_program: Res<Program>,
  discs: Tess,
  disc_vert_nb: usize,
  text_program: Res<Program>,
  text_quad: Tess,
  overlay_unit: OverlayUnit,
}

impl Renderer {
  pub fn new(w: u32, h: u32, max_tris: usize, max_discs: usize, scene: &mut Scene) -> Self {
    let fb = Framebuffer::new((w, h), 0).unwrap();

    let tri_program = scene.get("spectra/overlay/triangle.glsl", vec![]).unwrap();
    let tris = Tess::new(Mode::Triangle, TessVertices::Reserve::<Vert>(max_tris * 3), None);

    let disc_program = scene.get("spectra/overlay/disc.glsl", vec![DISC_SCREEN_RATIO.sem("ratio")]).unwrap();
    let discs = Tess::new(Mode::Point, TessVertices::Reserve::<Disc>(max_discs), None);

    let text_program = scene.get("spectra/overlay/text.glsl", vec![
      TEXT_SAMPLER.sem("text_texture"),
      TEXT_POS.sem("pos"),
      TEXT_SIZE.sem("size"),
      TEXT_SCALE.sem("scale"),
      TEXT_COLOR.sem("color")
    ]).unwrap();

    let text_quad = Tess::attributeless(Mode::TriangleStrip, 4);

    Renderer {
      framebuffer: fb,
      ratio: w as f32 / h as f32,
      tri_program: tri_program,
      tris: tris,
      tri_vert_nb: 0,
      disc_program: disc_program,
      discs: discs,
      disc_vert_nb: 0,
      text_program: text_program,
      text_quad: text_quad,
      overlay_unit: OverlayUnit::new(w, h)
    }
  }

  fn dispatch(&mut self, input: &RenderInput) {
    let mut tris = self.tris.as_slice_mut().unwrap();
    let mut tri_i = 0;
    let mut discs = self.discs.as_slice_mut().unwrap();
    let mut disc_i = 0;

    for &Triangle(a, b, c) in input.triangles {
      tris[tri_i] = a;
      tri_i += 1;

      tris[tri_i] = b;
      tri_i += 1;

      tris[tri_i] = c;
      tri_i += 1;
    }

    for disc in input.discs {
      discs[disc_i] = *disc;
      disc_i += 1;
    }

    self.tri_vert_nb = tri_i;
    self.disc_vert_nb = disc_i;
  }

  pub fn render(&mut self, input: RenderInput) -> &Texture2D {
    self.dispatch(&input);

    let tris = TessRender::one_sub(&self.tris, self.tri_vert_nb);
    let discs = TessRender::one_sub(&self.discs, self.disc_vert_nb);
    let text_quad = TessRender::one_whole(&self.text_quad);

    // FIXME: no alloc?
    let text_uniforms: Vec<_> = input.texts.map(|(texts, text_scale)| texts.iter().map(|text| {
      let (tex_w, tex_h) = text.text_texture.size();
      
      [
        TEXT_SAMPLER.alter(Unit::new(0)),
        TEXT_POS.alter(text.left_upper.pos),
        TEXT_SIZE.alter(self.overlay_unit.from_window(tex_w, tex_h)),
        TEXT_SCALE.alter(text_scale),
        TEXT_COLOR.alter(text.left_upper.color),
      ]
    }).collect()).unwrap_or(Vec::new());

    let text_textures: Vec<_> = input.texts.map(|(texts, _)| texts.iter().map(|text| [&***text.text_texture]).collect()).unwrap_or(Vec::new());

    let text_render_cmds: Vec<_> = input.texts.iter().enumerate().map(|(i, _)| {
        let blending = (Equation::Additive, Factor::One, Factor::SrcAlphaComplement);
        Pipe::empty()
          .uniforms(&text_uniforms[i])
          .textures(&text_textures[i])
          .unwrap(RenderCommand::new(Some(blending), true, vec![
            Pipe::new(text_quad.clone())
          ]))
      }).collect();

    let disc_uniforms = [
      DISC_SCREEN_RATIO.alter(self.ratio)
    ];

    Pipeline::new(&self.framebuffer, [0., 0., 0., 0.], &[], &[], vec![
      // render triangles
      Pipe::new(ShadingCommand::new(&self.tri_program.borrow(), vec![
        Pipe::new(RenderCommand::new(None, true, vec![
          Pipe::new(tris)
        ]))
      ])),
      // render discs
      Pipe::empty()
        .uniforms(&disc_uniforms)
        .unwrap(ShadingCommand::new(&self.disc_program.borrow(), vec![
          Pipe::new(RenderCommand::new(None, true, vec![
            Pipe::new(discs)
          ]))
        ])),
      // render texts
      Pipe::new(ShadingCommand::new(&self.text_program.borrow(), text_render_cmds))
    ]).run();

    &self.framebuffer.color_slot
  }
}

pub struct RenderInput<'a, 'b> where 'b: 'a {
  triangles: &'a [Triangle],
  discs: &'a [Disc],
  texts: Option<(&'a [Text<'b>], f32)>
}

impl<'a, 'b> RenderInput<'a, 'b> {
  pub fn new() -> Self {
    RenderInput {
      triangles: &[],
      discs: &[],
      texts: None,
    }
  }

  pub fn triangles(self, triangles: &'a [Triangle]) -> Self {
    RenderInput {
      triangles: triangles, ..self
    }
  }

  pub fn discs(self, discs: &'a [Disc]) -> Self {
    RenderInput {
      discs: discs,
      ..self
    }
  }

  pub fn texts(self, texts: &'a [Text<'b>], scale: f32) -> Self {
    RenderInput {
      texts: Some((texts, scale.max(0.))),
      ..self
    }
  }
}

/// Overlay units are used to position stuff in overlay in a universal way and screen independent
/// way. The screen uses normalized coordinates with the origin at lower left as the following
/// diagram:
///
/// ^ y (1)
/// |
/// |
/// |
/// O------> x (1)
#[derive(Copy, Clone, Debug, PartialEq)]
struct OverlayUnit {
  rw: f32,
  rh: f32
}

impl OverlayUnit {
  pub fn new(w: u32, h: u32) -> Self {
    OverlayUnit {
      rw: 1. / (w as f32),
      rh: 1. / (h as f32)
    }
  }

  pub fn from_window(&self, x: u32, y: u32) -> [f32; 2] {
    let x_ = x as f32 * self.rw;
    assert!(x_ >= 0. && x_ <= 1., "x={}", x_);

    let y_ = y as f32 * self.rh;
    assert!(y_ >= 0. && y_ <= 1., "y={}", y_);

    [x_, y_]
  }
}
