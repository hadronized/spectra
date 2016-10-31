use nalgebra::{ToHomogeneous, Unit, UnitQuaternion, Quaternion};
use serde::{Deserialize, Deserializer, Error, Serialize, Serializer};
use serde::de::{MapVisitor, Visitor};
use std::default::Default;

use luminance::linear::M44;
use luminance::shader::program::UniformUpdate;
use luminance_gl::gl33::Uniform;

pub use nalgebra::{Matrix4, Vector3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
  pub translation: Translation,
  pub orientation: Orientation,
  pub scale: Scale
}

impl Transform {
  pub fn new(translation: Translation, orientation: Orientation, scale: Scale) -> Self {
    Transform {
      translation: translation,
      orientation: orientation,
      scale: scale
    }
  }

  pub fn repos(self, pos: Position) -> Self {
    Transform { translation: pos, .. self }
  }

  pub fn translate(self, t: Translation) -> Self {
    Transform { translation: self.translation + t, .. self }
  }

  pub fn reorient(self, axis: Axis, phi: f32) -> Self {
    Transform { orientation: UnitQuaternion::from_axisangle(Unit::new(&axis), phi), .. self }
  }

  pub fn orient(self, axis: Axis, phi: f32) -> Self {
    Transform { orientation: UnitQuaternion::from_axisangle(Unit::new(&axis), phi) * self.orientation, .. self }
  }

  pub fn uni_scale(self, scale: f32) -> Self {
    Transform { scale: Scale { x: scale, y: scale, z: scale }, .. self }
  }

  pub fn rescale(self, scale: Scale) -> Self {
    Transform { scale: scale, .. self }
  }

  pub fn scale(self, scale: Scale) -> Self {
    let new_scale = Scale {
      x: self.scale.x * scale.x,
      y: self.scale.y * scale.y,
      z: self.scale.z * scale.z,
    };

    Transform { scale: new_scale, .. self }
  }

  pub fn to_inst_mat(&self) -> Matrix4<f32> {
    translation_matrix(self.translation) * self.scale.to_mat() * self.orientation.to_rotation_matrix().to_homogeneous()
  }

  pub fn to_view_mat(&self) -> Matrix4<f32> {
    self.orientation.to_rotation_matrix().to_homogeneous() * translation_matrix(self.translation) * self.scale.to_mat()
  }

  pub fn as_inst_uniform<'a>(u: Uniform<M44>) -> UniformUpdate<'a, Self> {
    let u: UniformUpdate<M44> = u.into();
    u.contramap(|transform: Transform| { *transform.to_inst_mat().as_ref() })
  }

  pub fn as_view_uniform<'a>(u: Uniform<M44>) -> UniformUpdate<'a, Self> {
    let u: UniformUpdate<M44> = u.into();
    u.contramap(|transform: Transform| { *transform.to_view_mat().as_ref() })
  }
}

impl Default for Transform {
  fn default() -> Self {
    Transform {
      translation: Vector3::new(0., 0., 0.),
      orientation: UnitQuaternion::from_unit_value_unchecked(Quaternion::from_parts(1., Vector3::new(0., 0., 0.))),
      scale: Scale::default()
    }
  }
}

impl Serialize for Transform {
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
    let mut struct_st = try!(serializer.serialize_struct("Transform", 3));

    try!(serializer.serialize_struct_elt(&mut struct_st, "translation", self.translation.as_ref()));
    try!(serializer.serialize_struct_elt(&mut struct_st, "orientation", self.orientation.as_ref().as_ref()));
    let scale = [self.scale.x, self.scale.y, self.scale.z];
    try!(serializer.serialize_struct_elt(&mut struct_st, "scale", scale));

    serializer.serialize_struct_end(struct_st)
  }
}

impl Deserialize for Transform {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    enum Field { Translation, Orientation, Scale };

    impl Deserialize for Field {
      fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error> where D: Deserializer {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
          type Value = Field;

          fn visit_str<E>(&mut self, value: &str) -> Result<Field, E> where E: Error {
            match value {
              "translation" => Ok(Field::Translation),
              "orientation" => Ok(Field::Orientation),
              "scale" => Ok(Field::Scale),
              _ => Err(Error::unknown_field(value))
            }
          }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
      }
    }

    struct TransformVisitor;

    impl Visitor for TransformVisitor {
      type Value = Transform;

      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut translation: Option<[f32; 3]> = None;
        let mut orientation: Option<[f32; 4]> = None;
        let mut scale: Option<[f32; 3]> = None;

        while let Some(key) = try!(visitor.visit_key::<Field>()) {
          match key {
            Field::Translation => {
              if translation.is_some() {
                return Err(<V::Error as Error>::duplicate_field("translation"));
              }

              translation = Some(try!(visitor.visit_value()));
            },
            Field::Orientation => {
              if orientation.is_some() {
                return Err(<V::Error as Error>::duplicate_field("orientation"));
              }

              orientation = Some(try!(visitor.visit_value()));
            },
            Field::Scale => {
              if scale.is_some() {
                return Err(<V::Error as Error>::duplicate_field("scale"));
              }

              scale = Some(try!(visitor.visit_value()));
            }
          }
        }

        try!(visitor.end());

        let translation = match translation {
          Some(a) => a,
          None => try!(visitor.missing_field("translation"))
        };

        let orientation = match orientation {
          Some(a) => a,
          None => try!(visitor.missing_field("orientation")),
        };

        let scale = match scale {
          Some(a) => a,
          None => try!(visitor.missing_field("scale"))
        };

        Ok(Transform::new(Translation::from(&translation), Orientation::new(&Quaternion::from(&orientation)), Scale::from(&scale)))
      }
    }

    const FIELDS: &'static [&'static str] = &["translation", "orientation", "scale"];
    deserializer.deserialize_struct("Transform", FIELDS, TransformVisitor)
  }
}

pub type Translation = Vector3<f32>;
pub type Axis = Vector3<f32>;
pub type Position = Vector3<f32>;
pub type Orientation = UnitQuaternion<f32>;

pub const X_AXIS: Axis = Axis { x: 1., y: 0., z: 0. };
pub const Y_AXIS: Axis = Axis { x: 0., y: 1., z: 0. };
pub const Z_AXIS: Axis = Axis { x: 0., y: 0., z: 1. };

/// Arbritrary scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Scale {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Scale {
      x: x,
      y: y,
      z: z
    }
  }

  pub fn uni(x: f32) -> Self {
    Scale {
      x: x,
      y: x,
      z: x
    }
  }

  pub fn to_mat(&self) -> Matrix4<f32> {
    Matrix4::new(
      self.x,     0.,     0., 0.,
          0., self.y,     0., 0.,
          0.,     0., self.z, 0.,
          0.,     0.,     0., 1.
    )
  }
}

impl Default for Scale {
  fn default() -> Self { Scale::new(1., 1., 1.) }
}

impl<'a> From<&'a [f32; 3]> for Scale {
  fn from(slice: &[f32; 3]) -> Self {
    Scale {
      x: slice[0],
      y: slice[1],
      z: slice[2]
    }
  }
}

fn translation_matrix(v: Translation) -> Matrix4<f32> {
  Matrix4::new(
    1., 0., 0., v.x,
    0., 1., 0., v.y,
    0., 0., 1., v.z,
    0., 0., 0.,  1.,
  )
}
