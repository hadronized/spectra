pub use luminance::pixel::{Depth32F, R32F, RGB32F, RGBA32F};
pub use luminance::texture::{Dim2, Flat, MagFilter, MinFilter, Sampler, Texture, TextureError, Wrap};
use image;
use image::ImageError;
use std::error::Error;
use std::fmt;
use std::ops::Deref;
use std::path::Path;

use sys::ignite::Ignite;
use sys::res::{FSKey, Load, Loaded, Storage};
use sys::res::helpers::{TyDesc, load_with};

// Common texture aliases.
pub type TextureRGB32F = Texture<Flat, Dim2, RGB32F>;
pub type TextureRGBA32F = Texture<Flat, Dim2, RGBA32F>;
pub type TextureR32F = Texture<Flat, Dim2, R32F>;
pub type TextureDepth32F = Texture<Flat, Dim2, Depth32F>;

/// Load an RGBA texture from an image at a path.
///
/// The `linearizer` argument is an option that gives the factor to apply to linearize if needed. Pass
/// `None` if the texture is already linearized.
pub fn load_rgba_texture<P>(
  ignite: &mut Ignite,
  path: P
) -> Result<TextureRGBA32F, TextureImageError>
where P: AsRef<Path> {
  info!("loading RGBA texture image: {:?}", path.as_ref());

  let img = image::open(path).map_err(TextureImageError::ParseFailed)?.flipv().to_rgba();
  let (w, h) = img.dimensions();
  let raw: Vec<f32> = img.into_raw().into_iter().map(|x| {
    x as f32 / 255.
  }).collect();

  let tex = Texture::new(ignite.surface(), [w, h], 0, &Sampler::default()).map_err(TextureImageError::ConversionFailed)?;
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

pub struct TextureImage(pub TextureRGBA32F);

impl Deref for TextureImage {
  type Target = TextureRGBA32F;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TyDesc for TextureImage {
  const TY_DESC: &'static str = "texture image";
}

impl Load<Ignite> for TextureImage {
  type Key = FSKey;

  type Error = TextureImageError;

  fn load(key: Self::Key, _: &mut Storage<Ignite>, ignite: &mut Ignite) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_path();

    load_with::<Self, _, _>(path, move || {
      load_rgba_texture(ignite, path).map(|rgba32f_tex| TextureImage(rgba32f_tex).into())
    })
  }

  impl_reload_passthrough!(Ignite);
}

#[derive(Debug)]
pub enum TextureImageError {
  ParseFailed(ImageError),
  ConversionFailed(TextureError)
}

impl fmt::Display for TextureImageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for TextureImageError {
  fn description(&self) -> &str {
    match *self {
      TextureImageError::ParseFailed(_) => "parse failed",
      TextureImageError::ConversionFailed(_) => "conversion failed"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      TextureImageError::ParseFailed(ref err) => Some(err),
      TextureImageError::ConversionFailed(ref err) => Some(err)
    }
  }
}
