pub use luminance::tess::{Mode, Tess, TessVertices};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use wavefront_obj::obj;

use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};
use scene::aabb::AABB;

/// A model tree representing the structure of a model.
///
/// It carries `Tess` on the leaves and 'AABB` on the nodes and leaves.
#[derive(Debug, PartialEq)]
pub enum ModelTree<V> {
  Leaf(AABB, Tess<V>),
  Node(AABB, Vec<ModelTree<V>>)
}

/// An OBJ model.
pub type ObjModel = ModelTree<ObjVertex>;

/// Vertex type used by OBJ models. It’s a triplet of vertex position, vertex normals and textures
/// coordinates.
pub type ObjVertex = (ObjVertexPos, ObjVertexNor, ObjVertexTexCoord);

pub type ObjVertexPos = [f32; 3];
pub type ObjVertexNor = [f32; 3];
pub type ObjVertexTexCoord = [f32; 2];

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObjModelKey(pub String);

impl ObjModelKey {
  pub fn new(key: &str) -> Self {
    ObjModelKey(key.to_owned())
  }
}

impl<'a> From<&'a str> for ObjModelKey {
  fn from(key: &str) -> Self {
    ObjModelKey::new(key)
  }
}

impl CacheKey for ObjModelKey {
  type Target = ObjModel;
}

impl StoreKey for ObjModelKey {
  fn key_to_path(&self) -> PathBuf {
    self.0.clone().into()
  }
}

impl Load for ObjModel {
  type Key = ObjModelKey;

  fn load(key: &Self::Key, _: &mut Store) -> Result<LoadResult<Self>, LoadError> {
    let path = key.key_to_path();

    let mut input = String::new();

    // load the data directly into memory; no buffering nor streaming
    {
      let mut file = File::open(&path).map_err(|_| LoadError::FileNotFound(path))?;
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
      let (vertices, indices, mode, aabb) = convert_geometry(geometry, &obj.vertices, &obj.normals, &obj.tex_vertices)?;
      let part = (aabb, Tess::new(mode, TessVertices::Fill(&vertices), &indices[..]));
      parts.push(part);
    }
  }

  let model_aabb = AABB::from_aabbs(parts.iter().map(|&(aabb, _)| aabb));
  let nodes = parts.into_iter().map(|(aabb, tess)| ModelTree::Leaf(aabb, tess)).collect();

  model_aabb.map(|aabb| ModelTree::Node(aabb, nodes)).ok_or(ModelError::NoGeometry)
}

// Convert wavefront_obj’s Geometry into a pair of vertices and indices.
//
// This function will regenerate the indices on the fly based on which are used in the shapes in the
// geometry. It’s used to create independent tessellation.
//
// It also provides the AABB enclosing the geometry.
fn convert_geometry(geo: &obj::Geometry, positions: &[obj::Vertex], normals: &[obj::Normal], tvertices: &[obj::TVertex]) -> Result<(Vec<ObjVertex>, Vec<u32>, Mode, AABB), ModelError> {
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

  AABB::from_vertices(vertices.iter().map(|v| v.0.into()))
    .map(|aabb| (vertices, indices, mode, aabb))
    .ok_or(ModelError::NoVertex)
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
  NoVertex,
  NoGeometry,
  NoShape
}

/// A material tree representing a possible interpretation of a model tree.
///
/// Only the leaves carry information about materials. The inner nodes are only used to match
/// against the model tree to represent.
pub enum MaterialTree<M> {
  Leaf(M),
  Node(Vec<MaterialTree<M>>)
}

impl<M> MaterialTree<M> {
  /// Traverse a model tree and represent it by zipping both trees to each other.
  ///
  /// If the zip is not total (partial zipping), non-matching nodes are just ignored.
  pub fn represent<V, F>(&self, model_tree: &ModelTree<V>, f: &mut F) where F: FnMut(&M, &Tess<V>) {
    match (self, model_tree) {
      (&MaterialTree::Leaf(ref material), &ModelTree::Leaf(_, ref tess)) => f(material, tess),

      (&MaterialTree::Node(ref material_nodes), &ModelTree::Node(_, ref model_nodes)) => {
        for (material, model) in material_nodes.iter().zip(model_nodes) {
          material.represent(model, f);
        }
      }

      _ => ()
    }
  }
}
