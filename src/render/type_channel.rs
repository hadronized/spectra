//! Type channel.
//!
//! A type channel is a number that represents the number of channel a type is associated with. For
//! instance, most people people might already be used to RGB and RGBA colors. They can for instance
//! be encoded as 4D-floating numbers. Type channels encode such information.

use std::fmt;
use serde::de::{self, Deserialize, Deserializer, Visitor, Unexpected};
use serde::ser::{Serialize, Serializer};

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
  fn deserialize_type_chan() {
    use serde_json::from_str;

    assert_eq!(from_str::<TypeChan>("1").unwrap(), TypeChan::One);
    assert_eq!(from_str::<TypeChan>("2").unwrap(), TypeChan::Two);
    assert_eq!(from_str::<TypeChan>("3").unwrap(), TypeChan::Three);
    assert_eq!(from_str::<TypeChan>("4").unwrap(), TypeChan::Four);
    assert!(from_str::<TypeChan>("5").is_err());
  }
}
