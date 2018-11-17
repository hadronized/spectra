//! Render input types and related functions.

use glsl::syntax::{ExternalDeclaration, StructFieldSpecifier, TypeSpecifier, TypeSpecifierNonArray};
use serde_derive::{Deserialize, Serialize};

use crate::render::type_channel::TypeChan;

/// Input type.
#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
  Int(TypeChan),
  #[serde(rename = "uint")]
  UInt(TypeChan),
  Float(TypeChan),
  Bool(TypeChan),
}

/// Role of an input. It can either be a functional input, like a vertex’s attribute, or a constant
/// parameter.
///
/// This is not really important when writing a shader code but becomes important when writing the
/// render pipeline.
#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
  Pipeline,
  Parameter,
  #[serde(rename = "built-in")]
  BuiltIn(BuiltIn)
}

/// Associate an input type to a given type.
pub trait InputType {
  const INPUT: Type;
}

/// Render inputs.
pub trait InputTypes {
  /// Get the types of the input.
  fn ty() -> &'static [Type];
}

impl<A> InputTypes for A where A: InputType {
  fn ty() -> &'static [Type] {
    &[A::INPUT]
  }
}

macro_rules! multi_input_type_impl {
  ($($t:tt),*) => {
    impl<$($t),*> InputTypes for ($($t),*) where $($t: InputType),* {
      fn ty() -> &'static [Type] {
        &[$($t::INPUT),*]
      }
    }
  }
}

multi_input_type_impl!(A, B);
multi_input_type_impl!(A, B, C);
multi_input_type_impl!(A, B, C, D);
multi_input_type_impl!(A, B, C, D, E);
multi_input_type_impl!(A, B, C, D, E, F);
multi_input_type_impl!(A, B, C, D, E, F, G);
multi_input_type_impl!(A, B, C, D, E, F, G, H);
multi_input_type_impl!(A, B, C, D, E, F, G, H, I);
multi_input_type_impl!(A, B, C, D, E, F, G, H, I, J);
multi_input_type_impl!(A, B, C, D, E, F, G, H, I, J, K);
multi_input_type_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

/// One-dimensional integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RI;

impl InputType for RI {
  const INPUT: Type = Type::Int(TypeChan::One);
}

/// Two dimensional integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGI;

impl InputType for RGI {
  const INPUT: Type = Type::Int(TypeChan::Two);
}

/// Three dimensional integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBI;

impl InputType for RGBI {
  const INPUT: Type = Type::Int(TypeChan::Three);
}

/// Four dimensional integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAI;

impl InputType for RGBAI {
  const INPUT: Type = Type::Int(TypeChan::Four);
}

/// One-dimensional unigned integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RU;

impl InputType for RU {
  const INPUT: Type = Type::UInt(TypeChan::One);
}

/// Two dimensional unsigned integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGU;

impl InputType for RGU {
  const INPUT: Type = Type::UInt(TypeChan::Two);
}

/// Three dimensional unsigned integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBU;

impl InputType for RGBU {
  const INPUT: Type = Type::UInt(TypeChan::Three);
}

/// Four dimensional unsigned integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAU;

impl InputType for RGBAU {
  const INPUT: Type = Type::UInt(TypeChan::Four);
}

/// One-dimensional floating output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RF;

impl InputType for RF {
  const INPUT: Type = Type::Float(TypeChan::One);
}

/// Two dimensional floating output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGF;

impl InputType for RGF {
  const INPUT: Type = Type::Float(TypeChan::Two);
}

/// Three dimensional floating output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBF;

impl InputType for RGBF {
  const INPUT: Type = Type::Float(TypeChan::Three);
}

/// Four dimensional floating output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAF;

impl InputType for RGBAF {
  const INPUT: Type = Type::Float(TypeChan::Four);
}

/// One-dimensional boolean output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RZ;

impl InputType for RZ {
  const INPUT: Type = Type::Bool(TypeChan::One);
}

/// Two dimensional boolean output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGZ;

impl InputType for RGZ {
  const INPUT: Type = Type::Bool(TypeChan::Two);
}

/// Three dimensional boolean output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBZ;

impl InputType for RGBZ {
  const INPUT: Type = Type::Bool(TypeChan::Three);
}

/// Four dimensional boolean output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAZ;

impl InputType for RGBAZ {
  const INPUT: Type = Type::Bool(TypeChan::Four);
}

/// Built-ins.
#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltIn {
  Time,
  FramebufferResolution
}

/// An input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Input {
  /// Name of the input.
  name: String,
  /// Type of the input.
  #[serde(rename = "type")]
  ty: Type,
}

impl Input {
  /// Create a new input.
  pub fn new<T, N>(name: N) -> Self
  where T: InputType,
        N: Into<String> {
    Input {
      name: name.into(),
      ty: T::INPUT,
    }
  }
}

/// Generate a GLSL structure given a list of inputs.
pub(crate) fn inputs_to_struct_decl<'a, I>(name: &str, inputs: I) -> ExternalDeclaration where I: IntoIterator<Item = &'a Input> {
  ExternalDeclaration::new_struct(name, inputs.into_iter().map(input_to_struct_field))
}

/// Generate a struct field from an input.
fn input_to_struct_field(input: &Input) -> StructFieldSpecifier {
  StructFieldSpecifier::new(input.name.as_str(), glsl_type_from_input_type(&input.ty))
}

/// Generate a GLSL type from a given input type.
fn glsl_type_from_input_type(ty: &Type) -> TypeSpecifier {
  let ty_nonarray = match *ty {
    Type::Int(TypeChan::One) => TypeSpecifierNonArray::Int,
    Type::Int(TypeChan::Two) => TypeSpecifierNonArray::IVec2,
    Type::Int(TypeChan::Three) => TypeSpecifierNonArray::IVec3,
    Type::Int(TypeChan::Four) => TypeSpecifierNonArray::IVec4,
    Type::UInt(TypeChan::One) => TypeSpecifierNonArray::UInt,
    Type::UInt(TypeChan::Two) => TypeSpecifierNonArray::UVec2,
    Type::UInt(TypeChan::Three) => TypeSpecifierNonArray::UVec3,
    Type::UInt(TypeChan::Four) => TypeSpecifierNonArray::UVec4,
    Type::Float(TypeChan::One) => TypeSpecifierNonArray::Float,
    Type::Float(TypeChan::Two) => TypeSpecifierNonArray::Vec2,
    Type::Float(TypeChan::Three) => TypeSpecifierNonArray::Vec3,
    Type::Float(TypeChan::Four) => TypeSpecifierNonArray::Vec4,
    Type::Bool(TypeChan::One) => TypeSpecifierNonArray::Bool,
    Type::Bool(TypeChan::Two) => TypeSpecifierNonArray::BVec2,
    Type::Bool(TypeChan::Three) => TypeSpecifierNonArray::BVec3,
    Type::Bool(TypeChan::Four) => TypeSpecifierNonArray::BVec4,
  };

  TypeSpecifier::new(ty_nonarray)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::{from_str, to_string};

  #[test]
  fn serialize_type() {
    assert_eq!(to_string(&RF::INPUT).unwrap(), r#"{"float":1}"#);
    assert_eq!(to_string(&RF::INPUT).unwrap(), r#"{"float":1}"#);
    assert_eq!(to_string(&RGF::INPUT).unwrap(), r#"{"float":2}"#);
    assert_eq!(to_string(&RGBF::INPUT).unwrap(), r#"{"float":3}"#);
    assert_eq!(to_string(&RGBAF::INPUT).unwrap(), r#"{"float":4}"#);
    assert_eq!(to_string(&RI::INPUT).unwrap(), r#"{"int":1}"#);
    assert_eq!(to_string(&RGI::INPUT).unwrap(), r#"{"int":2}"#);
    assert_eq!(to_string(&RGBI::INPUT).unwrap(), r#"{"int":3}"#);
    assert_eq!(to_string(&RGBAI::INPUT).unwrap(), r#"{"int":4}"#);
    assert_eq!(to_string(&RU::INPUT).unwrap(), r#"{"uint":1}"#);
    assert_eq!(to_string(&RGU::INPUT).unwrap(), r#"{"uint":2}"#);
    assert_eq!(to_string(&RGBU::INPUT).unwrap(), r#"{"uint":3}"#);
    assert_eq!(to_string(&RGBAU::INPUT).unwrap(), r#"{"uint":4}"#);
    assert_eq!(to_string(&RZ::INPUT).unwrap(), r#"{"bool":1}"#);
    assert_eq!(to_string(&RGZ::INPUT).unwrap(), r#"{"bool":2}"#);
    assert_eq!(to_string(&RGBZ::INPUT).unwrap(), r#"{"bool":3}"#);
    assert_eq!(to_string(&RGBAZ::INPUT).unwrap(), r#"{"bool":4}"#);
  }

  #[test]
  fn deserialize_type() {
    assert_eq!(from_str::<Type>(r#"{"float":1}"#).unwrap(), RF::INPUT);
    assert_eq!(from_str::<Type>(r#"{"float":2}"#).unwrap(), RGF::INPUT);
    assert_eq!(from_str::<Type>(r#"{"float":3}"#).unwrap(), RGBF::INPUT);
    assert_eq!(from_str::<Type>(r#"{"float":4}"#).unwrap(), RGBAF::INPUT);
    assert_eq!(from_str::<Type>(r#"{"int":1}"#).unwrap(), RI::INPUT);
    assert_eq!(from_str::<Type>(r#"{"int":2}"#).unwrap(), RGI::INPUT);
    assert_eq!(from_str::<Type>(r#"{"int":3}"#).unwrap(), RGBI::INPUT);
    assert_eq!(from_str::<Type>(r#"{"int":4}"#).unwrap(), RGBAI::INPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":1}"#).unwrap(), RU::INPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":2}"#).unwrap(), RGU::INPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":3}"#).unwrap(), RGBU::INPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":4}"#).unwrap(), RGBAU::INPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":1}"#).unwrap(), RZ::INPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":2}"#).unwrap(), RGZ::INPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":3}"#).unwrap(), RGBZ::INPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":4}"#).unwrap(), RGBAZ::INPUT);
  }

  #[test]
  fn serialize_role() {
    assert_eq!(&to_string(&Role::Pipeline).unwrap(), r#""pipeline""#);
    assert_eq!(&to_string(&Role::Parameter).unwrap(), r#""parameter""#);
  }

  #[test]
  fn deserialize_role() {
    assert_eq!(from_str::<Role>(r#""pipeline""#).unwrap(), Role::Pipeline);
    assert_eq!(from_str::<Role>(r#""parameter""#).unwrap(), Role::Parameter);
  }

  #[test]
  fn serialize_builtin() {
    assert_eq!(&to_string(&BuiltIn::Time).unwrap(), r#""time""#);
    assert_eq!(&to_string(&BuiltIn::FramebufferResolution).unwrap(), r#""framebuffer_resolution""#);
  }

  #[test]
  fn deserialize_builtin() {
    assert_eq!(from_str::<BuiltIn>(r#""time""#).unwrap(), BuiltIn::Time);
    assert_eq!(from_str::<BuiltIn>(r#""framebuffer_resolution""#).unwrap(), BuiltIn::FramebufferResolution);
  }

  #[test]
  fn input_construction() {
    let time = Input::new::<RF, _>("t");
    let jitter = Input::new::<RGBF, _>("jitter");

    assert_eq!(&time.name, "t");
    assert_eq!(time.ty, RF::INPUT);
    assert_eq!(&jitter.name, "jitter");
    assert_eq!(jitter.ty, RGBF::INPUT);
  }

  #[test]
  fn serialize_input() {
    let time = Input::new::<RF, _>("t");
    let jitter = Input::new::<RGBF, _>("jitter");

    assert_eq!(&to_string(&time).unwrap(), r#"{"name":"t","type":{"float":1}}"#);
    assert_eq!(&to_string(&jitter).unwrap(), r#"{"name":"jitter","type":{"float":3}}"#);
  }

  #[test]
  fn deserialize_input() {
    let time = Input::new::<RF, _>("t");
    let jitter = Input::new::<RGBF, _>("jitter");

    assert_eq!(from_str::<Input>(r#"{"name":"t","type":{"float":1}}"#).unwrap(), time);
    assert_eq!(from_str::<Input>(r#"{"name":"jitter","type":{"float":3}}"#).unwrap(), jitter);
  }
}
