use luminance::tess::{Mode, Tess, TessVertices};
use std::collections::BTreeMap;
use std::fs::File;
use std::fmt::{self, Debug, Formatter};
use std::io::Read;
use std::path::Path;
use std::vec;
use wavefront_obj::obj;

use resource::{Load, LoadError, LoadResult, ResCache};

/// A model is a tree of (possibly) nested tessellations.
pub enum Model<Vertex> {
  Tess(Tess<Vertex>),
  Parts(Vec<Model<Vertex>>)
}

impl<Vertex> Debug for Model<Vertex> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match *self {
      Model::Tess(_) => f.write_str("Tess"),
      Model::Parts(ref parts) => parts.fmt(f)
    }
  } 
}

impl<Vertex> From<Vec<Tess<Vertex>>> for Model<Vertex> {
  fn from(tess: Vec<Tess<Vertex>>) -> Self {
    Model::Parts(tess.into_iter().map(Model::Tess).collect())
  }
}

pub type ObjModel = Model<ObjVertex>;
pub type ObjVertex = (ObjVertexPos, ObjVertexNor, ObjVertexTexCoord);
pub type ObjVertexPos = [f32; 3];
pub type ObjVertexNor = [f32; 3];
pub type ObjVertexTexCoord = [f32; 2];

impl Load for ObjModel {
  type Args = ();

  const TY_STR: &'static str = "models";

  fn load<P>(path: P, _: &mut ResCache, _: Self::Args) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    let mut input = String::new();

    // load the data directly into memory; no buffering nor streaming
    {
      let mut file = File::open(path).map_err(|_| LoadError::FileNotFound(path.to_path_buf()))?;
      let _ = file.read_to_string(&mut input);
    }

    // parse the obj file and convert it
    let obj_set = obj::parse(input).map_err(|e| LoadError::ParseFailed(format!("{:?}", e)))?;

    convert_obj(obj_set).map_err(|e| LoadError::ConversionFailed(format!("{:?}", e))).map(Into::into)
  }
}

// Turn a wavefront obj object into a `Model`
fn convert_obj(obj_set: obj::ObjSet) -> Result<ObjModel, ModelError> {
  let mut parts = Vec::new();

  info!("{} objects to convert…", obj_set.objects.len());
  for obj in &obj_set.objects {
    info!("  converting {} geometries in object {}", obj.geometry.len(), obj.name);

    // convert all the geometries
    for geometry in &obj.geometry {
      info!("    {} vertices, {} normals, {} tex vertices", obj.vertices.len(), obj.normals.len(), obj.tex_vertices.len());
      let (vertices, indices, mode) = convert_geometry(geometry, &obj.vertices, &obj.normals, &obj.tex_vertices)?;
      let part = Tess::new(mode, TessVertices::Fill(&vertices), &indices[..]);
      parts.push(part);
    }
  }

  Ok(parts.into())
}

// Convert wavefront_obj’s Geometry into a pair of vertices and indices.
//
// This function will regenerate the indices on the fly based on which are used in the shapes in the
// geometry. It’s used to create independent tessellation.
fn convert_geometry(geo: &obj::Geometry, positions: &[obj::Vertex], normals: &[obj::Normal], tvertices: &[obj::TVertex]) -> Result<(Vec<ObjVertex>, Vec<u32>, Mode), ModelError> {
  if geo.shapes.is_empty() {
    return Err(ModelError::NoShape);
  }

  let mut vertices = Vec::new(); // FIXME: better allocation scheme?
  let mut indices = Vec::new();
  let mut index_map = BTreeMap::new();

  info!("    converting geometry");

  let mode = guess_mode(geo.shapes[0].primitive);

  for prim in geo.shapes.iter().map(|s| s.primitive) {
    let keys = create_keys_from_primitive(prim)?;

    for key in keys {
      match index_map.get(&key).cloned() {
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
      let a = vtnindex_to_key(i)?;
      Ok(vec![a])
    },
    obj::Primitive::Line(i, j) => {
      let a = vtnindex_to_key(i)?;
      let b = vtnindex_to_key(j)?;
      Ok(vec![a, b])
    },
    obj::Primitive::Triangle(i, j, k) => {
      let a = vtnindex_to_key(i)?;
      let b = vtnindex_to_key(j)?;
      let c = vtnindex_to_key(k)?;
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

fn interleave_vertex(p: &obj::Vertex, n: &obj::Normal, t: Option<&obj::TVertex>) -> ObjVertex {
  (convert_vertex(p), convert_nor(n), t.map_or([0., 0.], convert_tvertex))
}

fn convert_vertex(v: &obj::Vertex) -> ObjVertexPos {
  [v.x as f32, v.y as f32, v.z as f32]
}

fn convert_nor(n: &obj::Normal) -> ObjVertexNor {
  convert_vertex(n)
}

fn convert_tvertex(t: &obj::TVertex) -> ObjVertexTexCoord {
  [t.u as f32, t.v as f32]
}

fn guess_mode(prim: obj::Primitive) -> Mode {
  match prim {
    obj::Primitive::Point(_) => Mode::Point,
    obj::Primitive::Line(_, _) => Mode::Line,
    obj::Primitive::Triangle(_, _, _) => Mode::Triangle
  }
}

#[derive(Debug)]
pub enum ModelError {
  UnsupportedVertex,
  NoShape
}
