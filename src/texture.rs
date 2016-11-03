use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
use std::ops::Deref;
use std::path::Path;

pub use luminance::RGBA32F;

use resource::{Load, LoadError, Reload};

/// Load an RGBA texture from an image at a path.
pub fn load_rgba_texture<P>(path: P, sampler: &Sampler, linear: bool) -> ImageResult<Texture<Flat, Dim2, RGBA32F>> where P: AsRef<Path> {
  info!("loading texture image: \x1b[35m{:?}", path.as_ref());

  let image = try!(image::open(path)).to_rgba();
  let dim = image.dimensions();
  let raw: Vec<f32> = image.into_raw().into_iter().map(|x| {
    let y = x as f32 / 255.;

    if linear {
      y
    } else {
      y.powf(1. / 2.2)
    }
  }).collect();

  let tex = Texture::new(dim, 0, sampler);
  tex.upload_raw(false, &raw);

  Ok(tex)
}

/// Save an RGBA image on disk.
pub fn save_rgba_texture<P>(texture: &Texture<Flat, Dim2, RGBA32F>, path: P) where P: AsRef<Path> {
  info!("saving texture image to: \x1b[35m{:?}", path.as_ref());

  let texels = texture.get_raw_texels();
  let (w, h) = texture.size;
  let mut output = Vec::with_capacity((w * h) as usize);

  for texel in &texels {
    output.push((texel * 255.) as u8);
  }

  let _ = image::save_buffer(path, &output, w, h, image::ColorType::RGBA(8));
}

pub struct TextureImage {
  pub texture: Texture<Flat, Dim2, RGBA32F>,
  sampler: Sampler,
  linear: bool,
}

impl Deref for TextureImage {
  type Target = Texture<Flat, Dim2, RGBA32F>;

  fn deref(&self) -> &Self::Target {
    &self.texture
  }
}

impl Load for TextureImage {
  type Args = (Sampler, bool);

  fn load<P>(path: P, (sampler, linear): Self::Args) -> Result<Self, LoadError> where P: AsRef<Path> {
    load_rgba_texture(path, &sampler, linear)
      .map(|tex| TextureImage {
        texture: tex,
        sampler: sampler,
        linear: linear
        })
      .map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))
  }
}

impl Reload for TextureImage {
  fn reload<P>(&self, path :P) -> Result<Self, LoadError> where P: AsRef<Path> {
    Self::load(path, (self.sampler, self.linear))
  }
}
