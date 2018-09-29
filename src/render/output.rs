//! Render output types and related functions.

/// Output types.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Type {
  Int(TypeChan),
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

pub enum OutputList {
  Single(Type),
  Multi(Type, Box<OutputList>)
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
  const OUTPUT: Type = Type::Int(TypeChan::One);
}

/// Two dimensional unsigned integral output a.k.a. red-green channels.
pub struct RGU;

impl Output for RGU {
  const OUTPUT: Type = Type::Int(TypeChan::Two);
}

/// Three dimensional unsigned integral output a.k.a. red-green-blue channels.
pub struct RGBU;

impl Output for RGBU {
  const OUTPUT: Type = Type::Int(TypeChan::Three);
}

/// Four dimensional unsigned integral output a.k.a. red-green-blue-alpha channels.
pub struct RGBAU;

impl Output for RGBAU {
  const OUTPUT: Type = Type::Int(TypeChan::Four);
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
  const OUTPUT: Type = Type::Float(TypeChan::One);
}

/// Two dimensional boolean output a.k.a. red-green channels.
pub struct RGZ;

impl Output for RGZ {
  const OUTPUT: Type = Type::Float(TypeChan::Two);
}

/// Three dimensional boolean output a.k.a. red-green-blue channels.
pub struct RGBZ;

impl Output for RGBZ {
  const OUTPUT: Type = Type::Float(TypeChan::Three);
}

/// Four dimensional boolean output a.k.a. red-green-blue-alpha channels.
pub struct RGBAZ;

impl Output for RGBAZ {
  const OUTPUT: Type = Type::Float(TypeChan::Four);
}
