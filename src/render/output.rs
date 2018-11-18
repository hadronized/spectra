//! Render output types and related functions.

use glsl::syntax::{ExternalDeclaration, StructFieldSpecifier, TypeSpecifier, TypeSpecifierNonArray};
use serde_derive::{Deserialize, Serialize};

use crate::render::type_channel::TypeChan;

/// Output types.
#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
  Int(TypeChan),
  #[serde(rename = "uint")]
  UInt(TypeChan),
  Float(TypeChan),
  Bool(TypeChan)
}

/// Associate an output type to a given type.
pub trait OutputType {
  const OUTPUT: Type;
}

/// Render outputs.
pub trait OutputTypes {
  /// Get the types of the output.
  fn ty() -> &'static [Type];
}

impl<A> OutputTypes for A where A: OutputType {
  fn ty() -> &'static [Type] {
    &[A::OUTPUT]
  }
}

macro_rules! multi_output_type_impl {
  ($($t:tt),*) => {
    impl<$($t),*> OutputTypes for ($($t),*) where $($t: OutputType),* {
      fn ty() -> &'static [Type] {
        &[$($t::OUTPUT),*]
      }
    }
  }
}

multi_output_type_impl!(A, B);
multi_output_type_impl!(A, B, C);
multi_output_type_impl!(A, B, C, D);
multi_output_type_impl!(A, B, C, D, E);
multi_output_type_impl!(A, B, C, D, E, F);
multi_output_type_impl!(A, B, C, D, E, F, G);
multi_output_type_impl!(A, B, C, D, E, F, G, H);
multi_output_type_impl!(A, B, C, D, E, F, G, H, I);
multi_output_type_impl!(A, B, C, D, E, F, G, H, I, J);
multi_output_type_impl!(A, B, C, D, E, F, G, H, I, J, K);
multi_output_type_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

/// An output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Output {
  /// Name of the output.
  name: String,
  /// Type of the output.
  #[serde(rename = "type")]
  ty: Type
}

impl Output {
  /// Create a new output.
  pub fn new<T, N>(name: N) -> Self
  where T: OutputType,
        N: Into<String> {
    Output {
      name: name.into(),
      ty: T::OUTPUT
    }
  }
}

/// Generate a GLSL structure given a list of outputs.
pub(crate) fn outputs_to_struct_decl<'a, O>(
  name : &str,
  outputs: O
) -> ExternalDeclaration
where O: IntoIterator<Item = &'a Output> {
  ExternalDeclaration::new_struct(name, outputs.into_iter().map(output_to_struct_field))
}

/// Generate a struct field from an ouput.
fn output_to_struct_field(output: &Output) -> StructFieldSpecifier {
  StructFieldSpecifier::new(output.name.as_str(), glsl_type_from_output_type(&output.ty))
}

/// Generate a GLSL type from a given output type.
fn glsl_type_from_output_type(ty: &Type) -> TypeSpecifier {
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
  use glsl_quasiquote::glsl;
  use serde_json::{from_str, to_string};

  use crate::render::types::*;
  use super::*;

  #[test]
  fn serialize_type() {
    assert_eq!(to_string(&RF::OUTPUT).unwrap(), r#"{"float":1}"#);
    assert_eq!(to_string(&RGF::OUTPUT).unwrap(), r#"{"float":2}"#);
    assert_eq!(to_string(&RGBF::OUTPUT).unwrap(), r#"{"float":3}"#);
    assert_eq!(to_string(&RGBAF::OUTPUT).unwrap(), r#"{"float":4}"#);
    assert_eq!(to_string(&RI::OUTPUT).unwrap(), r#"{"int":1}"#);
    assert_eq!(to_string(&RGI::OUTPUT).unwrap(), r#"{"int":2}"#);
    assert_eq!(to_string(&RGBI::OUTPUT).unwrap(), r#"{"int":3}"#);
    assert_eq!(to_string(&RGBAI::OUTPUT).unwrap(), r#"{"int":4}"#);
    assert_eq!(to_string(&RU::OUTPUT).unwrap(), r#"{"uint":1}"#);
    assert_eq!(to_string(&RGU::OUTPUT).unwrap(), r#"{"uint":2}"#);
    assert_eq!(to_string(&RGBU::OUTPUT).unwrap(), r#"{"uint":3}"#);
    assert_eq!(to_string(&RGBAU::OUTPUT).unwrap(), r#"{"uint":4}"#);
    assert_eq!(to_string(&RZ::OUTPUT).unwrap(), r#"{"bool":1}"#);
    assert_eq!(to_string(&RGZ::OUTPUT).unwrap(), r#"{"bool":2}"#);
    assert_eq!(to_string(&RGBZ::OUTPUT).unwrap(), r#"{"bool":3}"#);
    assert_eq!(to_string(&RGBAZ::OUTPUT).unwrap(), r#"{"bool":4}"#);
  }

  #[test]
  fn deserialize_type() {
    assert_eq!(from_str::<Type>(r#"{"float":1}"#).unwrap(), RF::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"float":2}"#).unwrap(), RGF::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"float":3}"#).unwrap(), RGBF::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"float":4}"#).unwrap(), RGBAF::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"int":1}"#).unwrap(), RI::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"int":2}"#).unwrap(), RGI::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"int":3}"#).unwrap(), RGBI::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"int":4}"#).unwrap(), RGBAI::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":1}"#).unwrap(), RU::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":2}"#).unwrap(), RGU::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":3}"#).unwrap(), RGBU::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"uint":4}"#).unwrap(), RGBAU::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":1}"#).unwrap(), RZ::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":2}"#).unwrap(), RGZ::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":3}"#).unwrap(), RGBZ::OUTPUT);
    assert_eq!(from_str::<Type>(r#"{"bool":4}"#).unwrap(), RGBAZ::OUTPUT);
  }

  #[test]
  fn outputs_to_glsl_struct() {
    let color = Output::new::<RGBAF, _>("color");
    let depth = Output::new::<Float, _>("depth");
    let outputs = &[color, depth];

    let ed = vec![outputs_to_struct_decl("Output", outputs)];
    let expected = glsl!{
      struct Output {
        vec4 color;
        float depth;
      };
    };

    assert_eq!(ed, expected);
  }
}
