/// A 3-channel (red, green, blue) color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RGB {
  pub r: f32,
  pub g: f32,
  pub b: f32
}

impl RGB {
  pub fn new(r: f32, g: f32, b: f32) -> Self {
    RGB {
      r: r,
      g: g,
      b: b
    }
  }
}

impl From<[f32; 3]> for RGB {
  fn from([r, g, b]: [f32; 3]) -> Self {
    RGB::new(r, g, b)
  }
}

impl<'a> From<&'a [f32; 3]> for RGB {
  fn from(&[r, g, b]: &[f32; 3]) -> Self {
    RGB::new(r, g, b)
  }
}

impl From<RGB> for [f32; 3] {
  fn from(rgb: RGB) -> Self {
    [rgb.r, rgb.g, rgb.b]
  }
}

impl<'a> From<&'a RGB> for [f32; 3] {
  fn from(rgb: &RGB) -> Self {
    [rgb.r, rgb.g, rgb.b]
  }
}

/// A 4-channel (red, green, blue, alpha) color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RGBA {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl RGBA {
  pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
    RGBA {
      r: r,
      g: g,
      b: b,
      a: a
    }
  }
}

impl From<[f32; 4]> for RGBA {
  fn from([r, g, b, a]: [f32; 4]) -> Self {
    RGBA::new(r, g, b, a)
  }
}

impl<'a> From<&'a [f32; 4]> for RGBA {
  fn from(&[r, g, b, a]: &[f32; 4]) -> Self {
    RGBA::new(r, g, b, a)
  }
}

impl From<RGBA> for [f32; 4] {
  fn from(rgba: RGBA) -> Self {
    [rgba.r, rgba.g, rgba.b, rgba.a]
  }
}

impl<'a> From<&'a RGBA> for [f32; 4] {
  fn from(rgba: &RGBA) -> Self {
    [rgba.r, rgba.g, rgba.b, rgba.a]
  }
}
