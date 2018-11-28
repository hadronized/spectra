//! Common types used as input and output.

use crate::render::input::{self, InputType};
use crate::render::output::{self, OutputType};
use crate::render::type_channel::TypeChan;

/// One-dimensional integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Float;

impl InputType for Float {
  const INPUT: input::Type = input::Type::Float(TypeChan::One);
}

impl OutputType for Float {
  const OUTPUT: output::Type = output::Type::Float(TypeChan::One);
}

/// One-dimensional integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RI;

impl InputType for RI {
  const INPUT: input::Type = input::Type::Int(TypeChan::One);
}

impl OutputType for RI {
  const OUTPUT: output::Type = output::Type::Int(TypeChan::One);
}

/// Two dimensional integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGI;

impl InputType for RGI {
  const INPUT: input::Type = input::Type::Int(TypeChan::Two);
}

impl OutputType for RGI {
  const OUTPUT: output::Type = output::Type::Int(TypeChan::Two);
}

/// Three dimensional integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBI;

impl InputType for RGBI {
  const INPUT: input::Type = input::Type::Int(TypeChan::Three);
}

impl OutputType for RGBI {
  const OUTPUT: output::Type = output::Type::Int(TypeChan::Three);
}

/// Four dimensional integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAI;

impl InputType for RGBAI {
  const INPUT: input::Type = input::Type::Int(TypeChan::Four);
}

impl OutputType for RGBAI {
  const OUTPUT: output::Type = output::Type::Int(TypeChan::Four);
}

/// One-dimensional unigned integral output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RU;

impl InputType for RU {
  const INPUT: input::Type = input::Type::UInt(TypeChan::One);
}

impl OutputType for RU {
  const OUTPUT: output::Type = output::Type::UInt(TypeChan::One);
}

/// Two dimensional unsigned integral output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGU;

impl InputType for RGU {
  const INPUT: input::Type = input::Type::UInt(TypeChan::Two);
}

impl OutputType for RGU {
  const OUTPUT: output::Type = output::Type::UInt(TypeChan::Two);
}

/// Three dimensional unsigned integral output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBU;

impl InputType for RGBU {
  const INPUT: input::Type = input::Type::UInt(TypeChan::Three);
}

impl OutputType for RGBU {
  const OUTPUT: output::Type = output::Type::UInt(TypeChan::Three);
}

/// Four dimensional unsigned integral output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAU;

impl InputType for RGBAU {
  const INPUT: input::Type = input::Type::UInt(TypeChan::Four);
}

impl OutputType for RGBAU {
  const OUTPUT: output::Type = output::Type::UInt(TypeChan::Four);
}

/// One-dimensional floating output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RF;

impl InputType for RF {
  const INPUT: input::Type = input::Type::Float(TypeChan::One);
}

impl OutputType for RF {
  const OUTPUT: output::Type = output::Type::Float(TypeChan::One);
}

/// Two dimensional floating output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGF;

impl InputType for RGF {
  const INPUT: input::Type = input::Type::Float(TypeChan::Two);
}

impl OutputType for RGF {
  const OUTPUT: output::Type = output::Type::Float(TypeChan::Two);
}

/// Three dimensional floating output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBF;

impl InputType for RGBF {
  const INPUT: input::Type = input::Type::Float(TypeChan::Three);
}

impl OutputType for RGBF {
  const OUTPUT: output::Type = output::Type::Float(TypeChan::Three);
}

/// Four dimensional floating output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAF;

impl InputType for RGBAF {
  const INPUT: input::Type = input::Type::Float(TypeChan::Four);
}

impl OutputType for RGBAF {
  const OUTPUT: output::Type = output::Type::Float(TypeChan::Four);
}

/// One-dimensional boolean output a.k.a. red channel.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RZ;

impl InputType for RZ {
  const INPUT: input::Type = input::Type::Bool(TypeChan::One);
}

impl OutputType for RZ {
  const OUTPUT: output::Type = output::Type::Bool(TypeChan::One);
}

/// Two dimensional boolean output a.k.a. red-green channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGZ;

impl InputType for RGZ {
  const INPUT: input::Type = input::Type::Bool(TypeChan::Two);
}

impl OutputType for RGZ {
  const OUTPUT: output::Type = output::Type::Bool(TypeChan::Two);
}

/// Three dimensional boolean output a.k.a. red-green-blue channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBZ;

impl InputType for RGBZ {
  const INPUT: input::Type = input::Type::Bool(TypeChan::Three);
}

impl OutputType for RGBZ {
  const OUTPUT: output::Type = output::Type::Bool(TypeChan::Three);
}

/// Four dimensional boolean output a.k.a. red-green-blue-alpha channels.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RGBAZ;

impl InputType for RGBAZ {
  const INPUT: input::Type = input::Type::Bool(TypeChan::Four);
}

impl OutputType for RGBAZ {
  const OUTPUT: output::Type = output::Type::Bool(TypeChan::Four);
}
