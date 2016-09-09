use luminance::tessellation;
use luminance_gl::gl33;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use wavefront_obj::{self, obj};

pub type Vertex = (VertexPos, VertexNor, VertexTexCoord);
pub type VertexPos = [f32; 3];
pub type VertexNor = [f32; 3];
pub type VertexTexCoord = [f32; 2];

pub fn load<P>(path: P, mode: tessellation::Mode) -> Result<gl33::Tessellation, TessellationError> where P: AsRef<Path> {
  let path = path.as_ref();

  info!("loading tessellation: \x1b[35m{:?}", path);

  let mut input = String::new();

  // load the data directly into memory; no buffering nor streaming
  {
    let mut file = try!(File::open(path).map_err(|e| TessellationError::FileNotFound(path.to_path_buf(), format!("{:?}", e))));
    let _ = file.read_to_string(&mut input);
  }

  // parse the obj file and convert it
  let obj_set = try!(obj::parse(input).map_err(TessellationError::ParseFailed));

  convert_obj(obj_set, mode)
}

// Turn a loaded wavefront obj object into a luminance tessellation.
fn convert_obj(obj_set: obj::ObjSet, mode: tessellation::Mode) -> Result<gl33::Tessellation, TessellationError> {
  if obj_set.objects.len() != 1 {
    return Err(TessellationError::MultiObjects);
  }

  let obj = &obj_set.objects[0];
  info!("  found object \x1b[35m{}", obj.name);

  // interleave the vertices
  let interleaved_vertices = interleave_vertices(&obj.vertices, &obj.normals, &obj.tex_vertices);

  Err(TessellationError::Error)
}

// wavefront_obj exposes its vertex positions, normals and texture coordinates as deinterleaved.
// This function gathers all the components and yields an interleaved vector of vertices.
fn interleave_vertices(pos: &[obj::Vertex], nor: &[obj::Normal], tex: &[obj::TVertex]) -> Vec<Vertex> {
  let mut interleaved_vertices = Vec::with_capacity(pos.len());

  for ((p, n), t) in pos.iter().zip(nor).zip(tex) {
    let vertex = (convert_vertex(p), convert_nor(n), convert_tvertex(t));
    interleaved_vertices.push(vertex);
  }

  interleaved_vertices
}

fn convert_vertex(v: &obj::Vertex) -> VertexPos {
  [v.x as f32, v.y as f32, v.z as f32]
}

fn convert_nor(n: &obj::Normal) -> VertexNor {
  convert_vertex(n)
}

fn convert_tvertex(t: &obj::TVertex) -> VertexTexCoord {
  [t.x as f32, t.y as f32]
}

mod hot {
  use super::{TessellationError, Vertex};

  use luminance::tessellation;
  use luminance_gl::gl33;
  use resource::ResourceManager;
  use std::ops::Deref;
  use std::path::{Path, PathBuf};

  pub struct Tessellation {
    path: PathBuf,
    mode: tessellation::Mode,
    tess: gl33::Tessellation
  }

  impl Tessellation {
    fn load<P>(manager: &mut ResourceManager, path: P, mode: tessellation::Mode) -> Result<Self, TessellationError> where P: AsRef<Path> {
      Err(TessellationError::Error)
    }
  }
  
  impl Deref for Tessellation {
    type Target = gl33::Tessellation;

    fn deref(&self) -> &Self::Target {
      &self.tess
    }
  }
}

pub enum TessellationError {
  FileNotFound(PathBuf, String),
  ParseFailed(wavefront_obj::ParseError),
  MultiObjects,
  Error
}
