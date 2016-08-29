use luminance::{Dim2, Flat, Sampler};
use luminance_gl::gl33::Texture;
use image::{self, ImageResult};
#[cfg(feature = "hot-texture")]
use notify::RecommendedWatcher;
#[cfg(feature = "hot-texture")]
use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::Path;
#[cfg(feature = "hot-texture")]
use std::path::PathBuf;
#[cfg(feature = "hot-texture")]
use std::sync::mpsc;
#[cfg(feature = "hot-texture")]
use std::sync::{Arc, Mutex};

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

#[cfg(feature = "hot-texture")]
pub struct TextureImageBuilder {
  watcher: RecommendedWatcher,
  receivers: Arc<Mutex<BTreeMap<PathBuf, mpsc::Sender<()>>>>
}
#[cfg(not(feature = "hot-texture"))]
pub struct TextureImageBuilder {}

#[cfg(feature = "hot-texture")]
pub struct WrappedTextureImage {
  rx: mpsc::Receiver<()>,
  texture: TextureImage<RGBA32F>,
  sampler: Sampler,
  linear: bool,
  path: PathBuf
}
#[cfg(not(feature = "hot-texture"))]
pub struct WrappedTextureImage {
  texture: TextureImage<RGBA32F>
}

impl WrappedTextureImage {
  #[cfg(feature = "hot-texture")]
  fn reload(&mut self) {
    let texture = load_rgba_texture(self.path.as_path(), &self.sampler, self.linear);

    match texture {
      Ok(texture) => {
        self.texture = texture;
      },
      Err(err) => {
        err!("reloading texture has failed: {:?}", err);
      }
    }
  }

  #[cfg(feature = "hot-texture")]
  pub fn sync(&mut self) {
    if self.rx.try_recv().is_ok() {
      self.reload();
    }
  }
  #[cfg(not(feature = "hot-texture"))]
  pub fn sync(&mut self) {}
}

impl Deref for WrappedTextureImage {
  type Target = TextureImage<RGBA32F>;

  fn deref(&self) -> &Self::Target {
    &self.texture
  }
}
