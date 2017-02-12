use rusttype::{Font, FontCollection, Scale, point};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use texture::{Dim2, Flat, R32F, Sampler, Texture};

pub type Result<A> = ::std::result::Result<A, FontError>;

#[derive(Debug)]
pub enum FontError {
  IncorrectPath(PathBuf),
  MultipleFonts,
  RasterizationFailed(String)
}

pub struct TextTexture {
  texture: Texture<Flat, Dim2, R32F>,
}

impl Deref for TextTexture {
  type Target = Texture<Flat, Dim2, R32F>;

  fn deref(&self) -> &Self::Target {
    &self.texture
  }
}

/// Rasterizer responsible of rasterizing text.
pub struct Rasterizer<'a> {
  font: Font<'a>,
}

impl<'a> Rasterizer<'a> {
  pub fn from_file<P>(font_path: P) -> Result<Self> where P: AsRef<Path> {
    let font_path = font_path.as_ref();
    let mut data = Vec::new();

    {
      let mut fh = File::open(font_path).map_err(|_| FontError::IncorrectPath(font_path.to_owned()))?;
      let _ = fh.read_to_end(&mut data);
    }

    let font = FontCollection::from_bytes(data).into_font().ok_or(FontError::MultipleFonts)?;

    Ok(Rasterizer {
      font: font
    })
  }

  pub fn rasterize(&self, text: &str, height: f32) -> Result<TextTexture> {
    // prepare the font hints
    let px_height = height.ceil() as usize;
    let scale = Scale::uniform(height);
    let v_metrics = self.font.v_metrics(scale);
    let offset = point(0., v_metrics.ascent);
    let glyphs: Vec<_> = self.font.layout(text, scale, offset).collect();
    let width = glyphs.iter().rev().filter_map(|glyph|
      glyph.pixel_bounding_box().map(|bb|
        bb.min.x as f32 + glyph.unpositioned().h_metrics().advance_width)).next().unwrap_or(0.).ceil() as usize;

    // rasterize the string into a buffer
    let mut texels = vec![0.; width * px_height];

    for glyph in glyphs {
      if let Some(bb) = glyph.pixel_bounding_box() {
        // not a control character or some shit; we can rasterize it!
        glyph.draw(|x, y, v| {
          let x = x as i32 + bb.min.x;
          let y = y as i32 + bb.min.y;

          // clipping test
          if x > 0 && x < width as i32 && y > 0 && y < height as i32 {
            let x = x as usize;
            let y = y as usize;
            texels[x + y * width] = v;
          }
        });
      }
    }

    // create the texture from the buffer
    let sampler = Sampler::default();
    let texture = Texture::new((width as u32, px_height as u32), 4, &sampler).map_err(|e|
      FontError::RasterizationFailed(format!("{:?}", e)))?;
    texture.upload(true, &texels);

    Ok(TextTexture {
      texture: texture
    })
  }
}
