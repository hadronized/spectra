//! Render output types and related functions.

use std::fmt;
use serde::de::{self, Deserialize, Deserializer, Visitor, Unexpected};
use serde::ser::{Serialize, Serializer};
use serde_derive::Serialize;

/// Output types.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
  Int(TypeChan),
  #[serde(rename = "uint")]
  UInt(TypeChan),
  Float(TypeChan),
  Bool(TypeChan)
}

/// Output type channels. Can be 1, 2, 3 or 4 channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TypeChan {
  One,
  Two,
  Three,
  Four
}

impl Serialize for TypeChan {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    match *self {
      TypeChan::One => serializer.serialize_u8(1),
      TypeChan::Two => serializer.serialize_u8(2),
      TypeChan::Three => serializer.serialize_u8(3),
      TypeChan::Four => serializer.serialize_u8(4),
    }
  }
}

impl<'de> Deserialize<'de> for TypeChan {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: Deserializer<'de> {
    struct V;

    impl<'de> Visitor<'de> for V {
      type Value = TypeChan;
        
      fn expecting(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("a valid type channel")
      }

      fn visit_u64<E>(self, x: u64) -> Result<Self::Value, E> where E: de::Error {
        match x {
          1 => Ok(TypeChan::One),
          2 => Ok(TypeChan::Two),
          3 => Ok(TypeChan::Three),
          4 => Ok(TypeChan::Four),
          x => Err(E::invalid_value(Unexpected::Unsigned(x as u64), &"1, 2, 3 or 4"))
        }
      }
    }

    deserializer.deserialize_u64(V)
  }
}

/// Associate an output type to a given type.
pub trait Output {
  const OUTPUT: Type;
}

/// Render outputs.
pub trait Outputs {
  /// Get the types of the output.
  fn ty() -> &'static [Type];
}

impl<A> Outputs for A where A: Output {
  fn ty() -> &'static [Type] {
    &[A::OUTPUT]
  }
}

macro_rules! multi_output_impl {
  ($($t:tt),*) => {
    impl<$($t),*> Outputs for ($($t),*) where $($t: Output),* {
      fn ty() -> &'static [Type] {
        &[$($t::OUTPUT),*]
      }
    }
  }
}

multi_output_impl!(A, B);
multi_output_impl!(A, B, C);
multi_output_impl!(A, B, C, D);
multi_output_impl!(A, B, C, D, E);
multi_output_impl!(A, B, C, D, E, F);
multi_output_impl!(A, B, C, D, E, F, G);
multi_output_impl!(A, B, C, D, E, F, G, H);
multi_output_impl!(A, B, C, D, E, F, G, H, I);
multi_output_impl!(A, B, C, D, E, F, G, H, I, J);
multi_output_impl!(A, B, C, D, E, F, G, H, I, J, K);
multi_output_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

/// One-dimensional integral output a.k.a. red channel.
pub struct RI;

impl Output for RI {
  const OUTPUT: Type = Type::Int(TypeChan::One);
}

/// Two dimensional integral output a.k.a. red-green channels.
pub struct RGI;

impl Output for RGI {
  const OUTPUT: Type = Type::Int(TypeChan::Two);
}

/// Three dimensional integral output a.k.a. red-green-blue channels.
pub struct RGBI;

impl Output for RGBI {
  const OUTPUT: Type = Type::Int(TypeChan::Three);
}

/// Four dimensional integral output a.k.a. red-green-blue-alpha channels.
pub struct RGBAI;

impl Output for RGBAI {
  const OUTPUT: Type = Type::Int(TypeChan::Four);
}

/// One-dimensional unigned integral output a.k.a. red channel.
pub struct RU;

impl Output for RU {
  const OUTPUT: Type = Type::UInt(TypeChan::One);
}

/// Two dimensional unsigned integral output a.k.a. red-green channels.
pub struct RGU;

impl Output for RGU {
  const OUTPUT: Type = Type::UInt(TypeChan::Two);
}

/// Three dimensional unsigned integral output a.k.a. red-green-blue channels.
pub struct RGBU;

impl Output for RGBU {
  const OUTPUT: Type = Type::UInt(TypeChan::Three);
}

/// Four dimensional unsigned integral output a.k.a. red-green-blue-alpha channels.
pub struct RGBAU;

impl Output for RGBAU {
  const OUTPUT: Type = Type::UInt(TypeChan::Four);
}

/// One-dimensional floating output a.k.a. red channel.
pub struct RF;

impl Output for RF {
  const OUTPUT: Type = Type::Float(TypeChan::One);
}

/// Two dimensional floating output a.k.a. red-green channels.
pub struct RGF;

impl Output for RGF {
  const OUTPUT: Type = Type::Float(TypeChan::Two);
}

/// Three dimensional floating output a.k.a. red-green-blue channels.
pub struct RGBF;

impl Output for RGBF {
  const OUTPUT: Type = Type::Float(TypeChan::Three);
}

/// Four dimensional floating output a.k.a. red-green-blue-alpha channels.
pub struct RGBAF;

impl Output for RGBAF {
  const OUTPUT: Type = Type::Float(TypeChan::Four);
}

/// One-dimensional boolean output a.k.a. red channel.
pub struct RZ;

impl Output for RZ {
  const OUTPUT: Type = Type::Bool(TypeChan::One);
}

/// Two dimensional boolean output a.k.a. red-green channels.
pub struct RGZ;

impl Output for RGZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Two);
}

/// Three dimensional boolean output a.k.a. red-green-blue channels.
pub struct RGBZ;

impl Output for RGBZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Three);
}

/// Four dimensional boolean output a.k.a. red-green-blue-alpha channels.
pub struct RGBAZ;

impl Output for RGBAZ {
  const OUTPUT: Type = Type::Bool(TypeChan::Four);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialize_type_chan() {
    use serde_json::to_string;

    assert_eq!(to_string(&TypeChan::One).unwrap(), "1");
    assert_eq!(to_string(&TypeChan::Two).unwrap(), "2");
    assert_eq!(to_string(&TypeChan::Three).unwrap(),"3");
    assert_eq!(to_string(&TypeChan::Four).unwrap(), "4");
  }

  #[test]
  fn serialize_type() {
    use serde_json::to_string;

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
  fn deserialize_type_chan() {
    use serde_json::from_str;

    assert_eq!(from_str::<TypeChan>("1").unwrap(), TypeChan::One);
    assert_eq!(from_str::<TypeChan>("2").unwrap(), TypeChan::Two);
    assert_eq!(from_str::<TypeChan>("3").unwrap(), TypeChan::Three);
    assert_eq!(from_str::<TypeChan>("4").unwrap(), TypeChan::Four);
    assert!(from_str::<TypeChan>("5").is_err());
  }

  #[test]
  fn deserialize_type() {
    use serde_json::from_str;

    assert_eq!(from_str::<TypeChan>("1").unwrap(), TypeChan::One);
    assert_eq!(from_str::<TypeChan>("2").unwrap(), TypeChan::Two);
    assert_eq!(from_str::<TypeChan>("3").unwrap(), TypeChan::Three);
    assert_eq!(from_str::<TypeChan>("4").unwrap(), TypeChan::Four);
  }
}
