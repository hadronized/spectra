
pub type Vertex = (VertexPos, VertexNor, VertexTexCoord);
pub type VertexPos = [f32; 3];
pub type VertexNor = [f32; 3];
pub type VertexTexCoord = [f32; 2];

mod hot {
  use super::Vertex;

  use luminance::tessellation;
  use luminance_gl::gl33;
  use std::ops::Deref;

  pub struct Tessellation {
    mode: tessellation::Mode,
    vertices: Vec<Vertex>,
    indices: Option<Vec<u32>>,
    tess: gl33::Tessellation
  }
  
  impl Deref for Tessellation {
    type Target = gl33::Tessellation;

    fn deref(&self) -> &Self::Target {
      &self.tess
    }
  }
}
