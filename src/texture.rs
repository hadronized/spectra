use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
use std::path::Path;

pub use luminance::RGBA32F;

pub type TextureImage<F> = Texture<Flat, Dim2, F>;

/// Load an RGBA texture from an image at a path.
pub fn load_rgba_texture<P>(path: P, sampler: &Sampler) -> ImageResult<TextureImage<RGBA32F>> where P: AsRef<Path> {
  let image = try!(image::open(path)).to_rgba();
  let dim = image.dimensions();
  let raw: Vec<f32> = image.into_raw().into_iter().map(|x| x as f32 / 255.).collect();

  let tex = Texture::new(dim, 0, sampler);
  tex.upload_raw(false, &raw);

  Ok(tex)
}
