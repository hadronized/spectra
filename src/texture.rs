pub use luminance::pixel::{Depth32F, R32F, RGBA32F};
pub use luminance::texture::{Dim2, Flat, MagFilter, MinFilter, Sampler, Texture, Unit, Wrap};
use image;
use std::ops::Deref;
use std::path::Path;

use resource::{Load, LoadError, Reload, ResCache, Result};

// Common texture aliases.
pub type TextureRGBA32F = Texture<Flat, Dim2, RGBA32F>;
pub type TextureDepth32F = Texture<Flat, Dim2, Depth32F>;

/// Load an RGBA texture from an image at a path.
///
/// The `linearizer` argument is an option that gives the factor to apply to linearize if needed. Pass
/// `None` if the texture is already linearized.
pub fn load_rgba_texture<P, L>(path: P, sampler: &Sampler, linearizer: L) -> Result<TextureRGBA32F> where P: AsRef<Path>, L: Into<Option<f32>> {
  info!("loading texture image: \x1b[35m{:?}", path.as_ref());

  let img = image::open(path).map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))?.flipv().to_rgba();
  let (w, h) = img.dimensions();
  let linearizer = linearizer.into();
  let raw: Vec<f32> = img.into_raw().into_iter().map(|x| {
    let y = x as f32 / 255.;
    linearizer.map_or(y, |factor| y.powf(1. / factor))
  }).collect();

  let tex = Texture::new([w, h], 0, sampler).map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))?;
  tex.upload_raw(false, &raw);

  Ok(tex)
}

/// Save an RGBA image on disk.
pub fn save_rgba_texture<P>(texture: &TextureRGBA32F, path: P) where P: AsRef<Path> {
  info!("saving texture image to: \x1b[35m{:?}", path.as_ref());

  let texels = texture.get_raw_texels();
  let [w, h] = texture.size();
  let mut output = Vec::with_capacity((w * h) as usize);

  for texel in &texels {
    output.push((texel * 255.) as u8);
  }

  let _ = image::save_buffer(path, &output, w, h, image::ColorType::RGBA(8));
}

pub struct TextureImage {
  pub texture: TextureRGBA32F,
  sampler: Sampler,
  linearizer: Option<f32>,
}

impl Deref for TextureImage {
  type Target = TextureRGBA32F;

  fn deref(&self) -> &Self::Target {
    &self.texture
  }
}

impl Load for TextureImage {
  type Args = (Sampler, Option<f32>);

  const TY_STR: &'static str = "textures";

  fn load<P>(path: P, _: &mut ResCache, (sampler, linearizer): Self::Args) -> Result<Self> where P: AsRef<Path> {
    load_rgba_texture(path, &sampler, linearizer)
      .map(|tex| TextureImage {
        texture: tex,
        sampler: sampler,
        linearizer: linearizer
      })
  }
}

impl Reload for TextureImage {
  fn reload_args(&self) -> Self::Args {
    (self.sampler, self.linearizer)
  }
}
