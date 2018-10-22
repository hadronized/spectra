//! Render blocks.
//!
//! Render blocks are logical rendering units that have inputs and outputs. Inputs can represent
//! vertex attributes, user-specified values, built-ins or previous blocks’ outputs.

use cheddar::Module;
use glsl::syntax::TranslationUnit;
use warmy::Res;

use crate::render::input::Input;
use crate::render::output::Output;

#[derive(Clone, Debug)]
struct Block {
  inputs: Vec<Input>,
  outputs: Vec<Output>,
  code: Res<Module>
}
