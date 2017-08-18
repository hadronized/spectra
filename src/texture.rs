pub use luminance::pixel::{Depth32F, R32F, RGBA32F};
pub use luminance::texture::{Dim2, Flat, MagFilter, MinFilter, Sampler, Texture, Wrap};
use image;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use resource::{CacheKey, Load, LoadError, LoadResult, Store};

// Common texture aliases.
pub type TextureRGBA32F = Texture<Flat, Dim2, RGBA32F>;
pub type TextureDepth32F = Texture<Flat, Dim2, Depth32F>;

/// Load an RGBA texture from an image at a path.
///
/// The `linearizer` argument is an option that gives the factor to apply to linearize if needed. Pass
/// `None` if the texture is already linearized.
pub fn load_rgba_texture<P>(path: P) -> Result<TextureRGBA32F, LoadError> where P: AsRef<Path> {
  let img = image::open(path).map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))?.flipv().to_rgba();
  let (w, h) = img.dimensions();
  let raw: Vec<f32> = img.into_raw().into_iter().map(|x| {
    x as f32 / 255.
  }).collect();

  let tex = Texture::new([w, h], 0, &Sampler::default()).map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))?;
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

#[derive(Debug)]
pub struct TextureImage(pub TextureRGBA32F);

impl Deref for TextureImage {
  type Target = TextureRGBA32F;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureKey(pub String);

impl CacheKey for TextureKey {
  type Target = TextureImage;
}

impl Load for TextureImage {
  type Key = TextureKey;

  fn key_to_path(key: &Self::Key) -> PathBuf {
    key.0.clone().into()
  }

  fn load<P>(path: P, _: &mut Store) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path> {
    let result = load_rgba_texture(path).map(TextureImage)?.into();
    Ok(result)
  }
}
