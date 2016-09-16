use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
use std::path::Path;

pub use luminance::RGBA32F;

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

#[derive(Clone, Debug)]
pub enum TextureError {
  LoadingFailed(String)
}

#[cfg(feature = "hot-resource")]
mod hot {
  use luminance::{Dim2, Flat, Sampler};
  use luminance_gl::gl33::Texture;
  use std::ops::Deref;
  use std::path::{Path, PathBuf};
  use std::sync::mpsc;

  use resource::ResourceManager;

  use super::*;

  pub struct TextureImage {
    rx: mpsc::Receiver<()>,
    last_update_time: Option<f64>,
    texture: Texture<Flat, Dim2, RGBA32F>,
    sampler: Sampler,
    linear: bool,
    path: PathBuf
  }

  impl TextureImage {
    pub fn load<P>(manager: &mut ResourceManager, path: P, sampler: &Sampler, linear: bool) -> Result<Self, TextureError> where P: AsRef<Path> {
      let path = path.as_ref();

      let tex = load_rgba_texture(path, sampler, linear);

      match tex {
        Ok(tex) => {
          // monitor that texture
          let (sx, rx) = mpsc::channel();

          manager.monitor(path, sx);

          Ok(TextureImage {
            rx: rx,
            last_update_time: None,
            texture: tex,
            sampler: *sampler,
            linear: linear,
            path: path.to_path_buf()
          })
        },
        Err(e) => {
          Err(TextureError::LoadingFailed(format!("{:?}", e)))
        }
      }
    }

    fn reload(&mut self) {
      let path = self.path.as_path();
      let tex = load_rgba_texture(path, &self.sampler, self.linear);

      match tex {
        Ok(tex) => {
          self.texture = tex;
          info!("reloaded texture {:?}", path);
        },
        Err(e) => {
          err!("reloading texture {:?} has failed: {:?}", path, e);
        }
      }
    }

    decl_sync_hot!();
  }

  impl Deref for TextureImage {
    type Target = Texture<Flat, Dim2, RGBA32F>;

    fn deref(&self) -> &Self::Target {
      &self.texture
    }
  }
}

#[cfg(not(feature = "hot-resource"))]
mod cold {
  use luminance::{Dim2, Flat, Sampler};
  use luminance_gl::gl33::Texture;
  use std::ops::Deref;
  use std::path::Path;

  use resource::ResourceManager;

  use super::*;

  pub struct TextureImage(Texture<Flat, Dim2, RGBA32F>);

  impl TextureImage {
    pub fn load<P>(_: &mut ResourceManager, path: P, sampler: &Sampler, linear: bool) -> Result<Self, TextureError> where P: AsRef<Path> {
      let tex = load_rgba_texture(path, sampler, linear);

      match tex {
        Ok(tex) => {
          Ok(TextureImage(tex))
        },
        Err(e) => {
          Err(TextureError::LoadingFailed(format!("{:?}", e)))
        }
      }
    }

    pub fn sync(&mut self) {}
  }

  impl Deref for TextureImage {
    type Target = Texture<Flat, Dim2, RGBA32F>;

    fn deref(&self) -> &Self::Target {
      &self.0
    }
  }
}

#[cfg(feature = "hot-resource")]
pub use self::hot::*;
#[cfg(not(feature = "hot-resource"))]
pub use self::cold::*;
