use luminance::tessellation;
use luminance_gl::gl33::Tessellation;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::iter::IntoIterator;
use std::path::{Path, PathBuf};
use std::vec;
use wavefront_obj::{self, obj};

// FIXME: implement materials
pub type Material = ();

pub type Vertex = (VertexPos, VertexNor, VertexTexCoord);
pub type VertexPos = [f32; 3];
pub type VertexNor = [f32; 3];
pub type VertexTexCoord = [f32; 2];

pub struct ModelBase<'a> {
  pub parts: Vec<Part<'a>>
}

impl<'a> ModelBase<'a> {
  pub fn from_parts(parts: Vec<Part<'a>>) -> Self {
    ModelBase {
      parts: parts
    }
  }
}

impl<'a> IntoIterator for ModelBase<'a> {
  type Item = Part<'a>;
  type IntoIter = vec::IntoIter<Part<'a>>;

  fn into_iter(self) -> Self::IntoIter {
    self.parts.into_iter()
  }
}

pub struct Part<'a> {
  pub tess: Tessellation,
  pub mat: Option<&'a Material>
}

impl<'a> Part<'a> {
  pub fn new(tess: Tessellation, mat: Option<&'a Material>) -> Self {
    Part {
      tess: tess,
      mat: mat
    }
  }
}

pub fn load<'a, P>(path: P) -> Result<ModelBase<'a>, ModelError> where P: AsRef<Path> {
  let path = path.as_ref();

  info!("loading model: \x1b[35m{:?}", path);

  let mut input = String::new();

  // load the data directly into memory; no buffering nor streaming
  {
    let mut file = try!(File::open(path).map_err(|e| ModelError::FileNotFound(path.to_path_buf(), format!("{:?}", e))));
    let _ = file.read_to_string(&mut input);
  }

  // parse the obj file and convert it
  let obj_set = try!(obj::parse(input).map_err(ModelError::ParseFailed));

  convert_obj(obj_set)
}

// Turn a wavefront obj object into a `Model`
fn convert_obj<'a>(obj_set: obj::ObjSet) -> Result<ModelBase<'a>, ModelError> {
  let mut parts = Vec::new();

  info!("{} objects to convert…", obj_set.objects.len());
  for obj in &obj_set.objects {
    info!("  converting {} geometries in object \x1b[35m{}", obj.geometry.len(), obj.name);

    // convert all the geometries
    for geometry in &obj.geometry {
      info!("    {} vertices, {} normals, {} tex vertices", obj.vertices.len(), obj.normals.len(), obj.tex_vertices.len());
      let (vertices, indices, mode) = try!(convert_geometry(geometry, &obj.vertices, &obj.normals, &obj.tex_vertices));
      let part = Part::new(Tessellation::new(mode, &vertices, Some(&indices)), None); // FIXME: material
      parts.push(part);
    }
  }

  Ok(ModelBase::from_parts(parts))
}

// Convert wavefront_obj’s Geometry into a pair of vertices and indices.
//
// This function will regenerate the indices on the fly based on which are used in the shapes in the
// geometry. It’s used to create independent tessellation.
fn convert_geometry(geo: &obj::Geometry, positions: &[obj::Vertex], normals: &[obj::Normal], tvertices: &[obj::TVertex]) -> Result<(Vec<Vertex>, Vec<u32>, tessellation::Mode), ModelError> {
  if geo.shapes.is_empty() {
    return Err(ModelError::NoShape);
  }

  let mut vertices = Vec::new(); // FIXME: better allocation scheme?
  let mut indices = Vec::new();
  let mut index_map = BTreeMap::new();

  info!("    converting geometry");

  let mode = guess_mode(geo.shapes[0].primitive);

  for prim in geo.shapes.iter().map(|s| s.primitive) {
    let keys = try!(create_keys_from_primitive(prim));

    for key in keys {
      match index_map.get(&key).map(|&i| i) {
        Some(index) => {
          // that triplet already exists; just append the index in the indices buffer
          indices.push(index);
        },
        None => {
          // this is a new, not yet discovered triplet; create the corresponding vertex and add it
          // to the vertices buffer, and map the triplet to the index in the indices buffer
          let vertex = interleave_vertex(&positions[key.0], &normals[key.1], key.2.map(|ki| &tvertices[ki]));
          let index = vertices.len() as u32;

          vertices.push(vertex);
          indices.push(index);
          index_map.insert(key, index);
        }
      }
    }
  }

  Ok((vertices, indices, mode))
}

// Create triplet keys from wavefront_obj primitives. If any primitive doesn’t have all the triplet
// information (position, normal, tex), a ModelError::UnsupportedVertex error is returned instead.
fn create_keys_from_primitive(prim: obj::Primitive) -> Result<Vec<(usize, usize, Option<usize>)>, ModelError> {
  match prim {
    obj::Primitive::Point(i) => {
      let a = try!(vtnindex_to_key(i));
      Ok(vec![a])
    },
    obj::Primitive::Line(i, j) => {
      let a = try!(vtnindex_to_key(i));
      let b = try!(vtnindex_to_key(j));
      Ok(vec![a, b])
    },
    obj::Primitive::Triangle(i, j, k) => {
      let a = try!(vtnindex_to_key(i));
      let b = try!(vtnindex_to_key(j));
      let c = try!(vtnindex_to_key(k));
      Ok(vec![a, b, c])
    }
  }
}

// Convert from a wavefront_obj VTNIndex into our triplet, raising error if not possible.
fn vtnindex_to_key(i: obj::VTNIndex) -> Result<(usize, usize, Option<usize>), ModelError> {
  match i {
    (pi, ti, Some(ni)) => Ok((pi, ni, ti)),
    _ => Err(ModelError::UnsupportedVertex)
  }
}

fn interleave_vertex(p: &obj::Vertex, n: &obj::Normal, t: Option<&obj::TVertex>) -> Vertex {
  (convert_vertex(p), convert_nor(n), t.map_or([0., 0.], convert_tvertex))
}

fn convert_vertex(v: &obj::Vertex) -> VertexPos {
  [v.x as f32, v.y as f32, v.z as f32]
}

fn convert_nor(n: &obj::Normal) -> VertexNor {
  convert_vertex(n)
}

fn convert_tvertex(t: &obj::TVertex) -> VertexTexCoord {
  [t.u as f32, t.v as f32]
}

fn guess_mode(prim: obj::Primitive) -> tessellation::Mode {
  match prim {
    obj::Primitive::Point(_) => tessellation::Mode::Point,
    obj::Primitive::Line(_, _) => tessellation::Mode::Line,
    obj::Primitive::Triangle(_, _, _) => tessellation::Mode::Triangle
  }
}

#[derive(Debug)]
pub enum ModelError {
  FileNotFound(PathBuf, String),
  ParseFailed(wavefront_obj::ParseError),
  MultiObjects,
  UnsupportedVertex,
  NoShape
}

#[cfg(feature = "hot-resource")]
mod hot {
  use std::ops::Deref;
  use std::path::{Path, PathBuf};
  use std::sync::mpsc;

  use super::{ModelBase, ModelError};

  use resource::ResourceManager;

  pub struct Model<'a> {
    rx: mpsc::Receiver<()>,
    last_update_time: Option<f64>,
    model: ModelBase<'a>,
    path: PathBuf
  }

  impl<'a> Model<'a> {
    pub fn load<P>(manager: &mut ResourceManager, path: P) -> Result<Self, ModelError> where P: AsRef<Path> {
      let path = path.as_ref();
      let model = try!(super::load(path));

      let (sx, rx) = mpsc::channel();
      manager.monitor(path, sx);

      Ok(Model {
        rx: rx,
        last_update_time: None,
        model: model,
        path: path.to_owned()
      })
    }

    fn reload(&mut self) {
      match super::load(self.path.as_path()) {
        Ok(model) => {
          self.model = model;
          info!("reloaded model");
        },
        Err(e) => {
          err!("reloading model has failed: {:?}", e);
        }
      }
    }

    decl_sync_hot!();
  }

  impl<'a> Deref for Model<'a> {
    type Target = ModelBase<'a>;

    fn deref(&self) -> &Self::Target {
      &self.model
    }
  }
}

#[cfg(not(feature = "hot-resource"))]
mod cold {
  use std::ops::Deref;
  use std::path::Path;

  use super::{ModelBase, ModelError};

  use resource::ResourceManager;

  pub struct Model<'a>(ModelBase<'a>);

  impl<'a> Model<'a> {
    pub fn load<P>(_: &mut ResourceManager, path: P) -> Result<Self, ModelError> where P: AsRef<Path> {
      super::load(path).map(|m| Model(m))
    }

    pub fn sync(&mut self) {
    }
  }

  impl<'a> Deref for Model<'a> {
    type Target = ModelBase<'a>;

    fn deref(&self) -> &Self::Target {
      &self.0
    }
  }
}

#[cfg(feature = "hot-resource")]
pub use self::hot::*;
#[cfg(not(feature = "hot-resource"))]
pub use self::cold::*;
