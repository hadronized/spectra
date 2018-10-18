//! Render input types and related functions.

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

/// The time built-in.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Time;

impl InputType for Time {
  const INPUT: Type = Type::BuiltIn(BuiltIn::Time);
}

/// The framebuffer resolution built-in.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct FramebufferResolution;

impl InputType for FramebufferResolution {
  const INPUT: Type = Type::BuiltIn(BuiltIn::FramebufferResolution);
}

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
#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Input {
  /// Name of the input.
  name: String,
  /// Type of the input.
  ty: Type
}

impl Input {
  /// Create a new input.
  fn new<T, N>(name: N) -> Self
  where T: InputType,
        N: Into<String> {
    Input {
      name: name.into(),
      ty: T::INPUT
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::{from_str, to_string};

  #[test]
  fn serialize_type() {
    assert_eq!(to_string(&RF::INPUT).unwrap(), "{\"float\":1}");
    assert_eq!(to_string(&RF::INPUT).unwrap(), "{\"float\":1}");
    assert_eq!(to_string(&RGF::INPUT).unwrap(), "{\"float\":2}");
    assert_eq!(to_string(&RGBF::INPUT).unwrap(), "{\"float\":3}");
    assert_eq!(to_string(&RGBAF::INPUT).unwrap(), "{\"float\":4}");
    assert_eq!(to_string(&RI::INPUT).unwrap(), "{\"int\":1}");
    assert_eq!(to_string(&RGI::INPUT).unwrap(), "{\"int\":2}");
    assert_eq!(to_string(&RGBI::INPUT).unwrap(), "{\"int\":3}");
    assert_eq!(to_string(&RGBAI::INPUT).unwrap(), "{\"int\":4}");
    assert_eq!(to_string(&RU::INPUT).unwrap(), "{\"uint\":1}");
    assert_eq!(to_string(&RGU::INPUT).unwrap(), "{\"uint\":2}");
    assert_eq!(to_string(&RGBU::INPUT).unwrap(), "{\"uint\":3}");
    assert_eq!(to_string(&RGBAU::INPUT).unwrap(), "{\"uint\":4}");
    assert_eq!(to_string(&RZ::INPUT).unwrap(), "{\"bool\":1}");
    assert_eq!(to_string(&RGZ::INPUT).unwrap(), "{\"bool\":2}");
    assert_eq!(to_string(&RGBZ::INPUT).unwrap(), "{\"bool\":3}");
    assert_eq!(to_string(&RGBAZ::INPUT).unwrap(), "{\"bool\":4}");
    assert_eq!(to_string(&Type::BuiltIn(BuiltIn::Time)).unwrap(), "{\"built_in\":\"time\"}");
    assert_eq!(to_string(&Type::BuiltIn(BuiltIn::FramebufferResolution)).unwrap(), "{\"built_in\":\"framebuffer_resolution\"}");
  }

  #[test]
  fn deserialize_type() {
    assert_eq!(from_str::<Type>("{\"float\":1}").unwrap(), RF::INPUT);
    assert_eq!(from_str::<Type>("{\"float\":2}").unwrap(), RGF::INPUT);
    assert_eq!(from_str::<Type>("{\"float\":3}").unwrap(), RGBF::INPUT);
    assert_eq!(from_str::<Type>("{\"float\":4}").unwrap(), RGBAF::INPUT);
    assert_eq!(from_str::<Type>("{\"int\":1}").unwrap(), RI::INPUT);
    assert_eq!(from_str::<Type>("{\"int\":2}").unwrap(), RGI::INPUT);
    assert_eq!(from_str::<Type>("{\"int\":3}").unwrap(), RGBI::INPUT);
    assert_eq!(from_str::<Type>("{\"int\":4}").unwrap(), RGBAI::INPUT);
    assert_eq!(from_str::<Type>("{\"uint\":1}").unwrap(), RU::INPUT);
    assert_eq!(from_str::<Type>("{\"uint\":2}").unwrap(), RGU::INPUT);
    assert_eq!(from_str::<Type>("{\"uint\":3}").unwrap(), RGBU::INPUT);
    assert_eq!(from_str::<Type>("{\"uint\":4}").unwrap(), RGBAU::INPUT);
    assert_eq!(from_str::<Type>("{\"bool\":1}").unwrap(), RZ::INPUT);
    assert_eq!(from_str::<Type>("{\"bool\":2}").unwrap(), RGZ::INPUT);
    assert_eq!(from_str::<Type>("{\"bool\":3}").unwrap(), RGBZ::INPUT);
    assert_eq!(from_str::<Type>("{\"bool\":4}").unwrap(), RGBAZ::INPUT);
    assert_eq!(from_str::<Type>("{\"built_in\":\"time\"}").unwrap(), Type::BuiltIn(BuiltIn::Time));
    assert_eq!(from_str::<Type>("{\"built_in\":\"framebuffer_resolution\"}").unwrap(), Type::BuiltIn(BuiltIn::FramebufferResolution));
  }

  #[test]
  fn serialize_builtin() {
    assert_eq!(&to_string(&BuiltIn::Time).unwrap(), "\"time\"");
    assert_eq!(&to_string(&BuiltIn::FramebufferResolution).unwrap(), "\"framebuffer_resolution\"");
  }

  #[test]
  fn deserialize_builtin() {
    assert_eq!(from_str::<BuiltIn>("\"time\"").unwrap(), BuiltIn::Time);
    assert_eq!(from_str::<BuiltIn>("\"framebuffer_resolution\"").unwrap(), BuiltIn::FramebufferResolution);
  }

  #[test]
  fn serialize_input() {
    let time = Input::new::<Time, _>("t");
    assert_eq!(&time.name, "t");
    assert_eq!(time.ty, Time::INPUT);

    let jitter = Input::new::<RGBF, _>("jitter");
    assert_eq!(&jitter.name, "jitter");
    assert_eq!(jitter.ty, RGBF::INPUT);
  }
}
