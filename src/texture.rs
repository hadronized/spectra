use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
use std::path::Path;

pub use luminance::RGBA32F;

pub type TextureImage<F> = Texture<Flat, Dim2, F>;

/// Load an RGBA texture from an image at a path.
pub fn load_rgba_texture<P>(path: P, sampler: &Sampler, linear: bool) -> ImageResult<TextureImage<RGBA32F>> where P: AsRef<Path> {
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
pub fn save_rgba_texture<P>(texture: &TextureImage<RGBA32F>, path: P) where P: AsRef<Path> {
  let texels = texture.get_raw_texels();
  let (w, h) = texture.size;
  let mut output = Vec::with_capacity((w * h) as usize);

  for texel in &texels {
    output.push((texel * 255.) as u8);
  }

  let _ = image::save_buffer(path, &output, w, h, image::ColorType::RGBA(8));
}
