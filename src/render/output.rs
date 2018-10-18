//! Render output types and related functions.

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

/// One-dimensional integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RI;

impl OutputType for RI {
  const OUTPUT: Type = Type::Int(TypeChan::One);
}

/// Two dimensional integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGI;

impl OutputType for RGI {
  const OUTPUT: Type = Type::Int(TypeChan::Two);
}

/// Three dimensional integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBI;

impl OutputType for RGBI {
  const OUTPUT: Type = Type::Int(TypeChan::Three);
}

/// Four dimensional integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAI;

impl OutputType for RGBAI {
  const OUTPUT: Type = Type::Int(TypeChan::Four);
}

/// One-dimensional unigned integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RU;

impl OutputType for RU {
  const OUTPUT: Type = Type::UInt(TypeChan::One);
}

/// Two dimensional unsigned integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGU;

impl OutputType for RGU {
  const OUTPUT: Type = Type::UInt(TypeChan::Two);
}

/// Three dimensional unsigned integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBU;

impl OutputType for RGBU {
  const OUTPUT: Type = Type::UInt(TypeChan::Three);
}

/// Four dimensional unsigned integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAU;

impl OutputType for RGBAU {
  const OUTPUT: Type = Type::UInt(TypeChan::Four);
}

/// One-dimensional floating output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RF;

impl OutputType for RF {
  const OUTPUT: Type = Type::Float(TypeChan::One);
}

/// Two dimensional floating output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGF;

impl OutputType for RGF {
  const OUTPUT: Type = Type::Float(TypeChan::Two);
}

/// Three dimensional floating output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBF;

impl OutputType for RGBF {
  const OUTPUT: Type = Type::Float(TypeChan::Three);
}

/// Four dimensional floating output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAF;

impl OutputType for RGBAF {
  const OUTPUT: Type = Type::Float(TypeChan::Four);
}

/// One-dimensional boolean output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RZ;

impl OutputType for RZ {
  const OUTPUT: Type = Type::Bool(TypeChan::One);
}

/// Two dimensional boolean output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGZ;

impl OutputType for RGZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Two);
}

/// Three dimensional boolean output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBZ;

impl OutputType for RGBZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Three);
}

/// Four dimensional boolean output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAZ;

impl OutputType for RGBAZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Four);
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::{from_str, to_string};

  #[test]
  fn serialize_type() {
    assert_eq!(to_string(&RF::OUTPUT).unwrap(), "{\"float\":1}");
    assert_eq!(to_string(&RGF::OUTPUT).unwrap(), "{\"float\":2}");
    assert_eq!(to_string(&RGBF::OUTPUT).unwrap(), "{\"float\":3}");
    assert_eq!(to_string(&RGBAF::OUTPUT).unwrap(), "{\"float\":4}");
    assert_eq!(to_string(&RI::OUTPUT).unwrap(), "{\"int\":1}");
    assert_eq!(to_string(&RGI::OUTPUT).unwrap(), "{\"int\":2}");
    assert_eq!(to_string(&RGBI::OUTPUT).unwrap(), "{\"int\":3}");
    assert_eq!(to_string(&RGBAI::OUTPUT).unwrap(), "{\"int\":4}");
    assert_eq!(to_string(&RU::OUTPUT).unwrap(), "{\"uint\":1}");
    assert_eq!(to_string(&RGU::OUTPUT).unwrap(), "{\"uint\":2}");
    assert_eq!(to_string(&RGBU::OUTPUT).unwrap(), "{\"uint\":3}");
    assert_eq!(to_string(&RGBAU::OUTPUT).unwrap(), "{\"uint\":4}");
    assert_eq!(to_string(&RZ::OUTPUT).unwrap(), "{\"bool\":1}");
    assert_eq!(to_string(&RGZ::OUTPUT).unwrap(), "{\"bool\":2}");
    assert_eq!(to_string(&RGBZ::OUTPUT).unwrap(), "{\"bool\":3}");
    assert_eq!(to_string(&RGBAZ::OUTPUT).unwrap(), "{\"bool\":4}");
  }

  #[test]
  fn deserialize_type() {
    assert_eq!(from_str::<Type>("{\"float\":1}").unwrap(), RF::OUTPUT);
    assert_eq!(from_str::<Type>("{\"float\":2}").unwrap(), RGF::OUTPUT);
    assert_eq!(from_str::<Type>("{\"float\":3}").unwrap(), RGBF::OUTPUT);
    assert_eq!(from_str::<Type>("{\"float\":4}").unwrap(), RGBAF::OUTPUT);
    assert_eq!(from_str::<Type>("{\"int\":1}").unwrap(), RI::OUTPUT);
    assert_eq!(from_str::<Type>("{\"int\":2}").unwrap(), RGI::OUTPUT);
    assert_eq!(from_str::<Type>("{\"int\":3}").unwrap(), RGBI::OUTPUT);
    assert_eq!(from_str::<Type>("{\"int\":4}").unwrap(), RGBAI::OUTPUT);
    assert_eq!(from_str::<Type>("{\"uint\":1}").unwrap(), RU::OUTPUT);
    assert_eq!(from_str::<Type>("{\"uint\":2}").unwrap(), RGU::OUTPUT);
    assert_eq!(from_str::<Type>("{\"uint\":3}").unwrap(), RGBU::OUTPUT);
    assert_eq!(from_str::<Type>("{\"uint\":4}").unwrap(), RGBAU::OUTPUT);
    assert_eq!(from_str::<Type>("{\"bool\":1}").unwrap(), RZ::OUTPUT);
    assert_eq!(from_str::<Type>("{\"bool\":2}").unwrap(), RGZ::OUTPUT);
    assert_eq!(from_str::<Type>("{\"bool\":3}").unwrap(), RGBZ::OUTPUT);
    assert_eq!(from_str::<Type>("{\"bool\":4}").unwrap(), RGBAZ::OUTPUT);
  }
}
