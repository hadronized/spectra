//! Render blocks.
//!
//! Render blocks are logical rendering units that have inputs and outputs. Inputs can represent
//! vertex attributes, user-specified values, built-ins or previous blocks’ outputs.

use glsl::syntax::TranslationUnit;
use warmy::Res;

use crate::render::input::Input;
use crate::render::output::Output;

type GLSL = TranslationUnit;

#[derive(Clone, Debug)]
struct Block {
  inputs: Vec<Input>,
  outputs: Vec<Output>,
  code: Res<GLSL>
}

#[cfg(test)]
mod tests {
}
