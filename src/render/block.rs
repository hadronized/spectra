//! Render combinatory blocks.
//!
//! Render blocks are logical rendering units that have inputs and outputs. Inputs can represent
//! vertex attributes, user-specified values, built-ins or previous blocks’ outputs.

use glsl::syntax::ExternalDeclaration;
use std::iter::once;

use crate::render::input::{Input, inputs_to_struct_decl};
use crate::render::output::{Output, outputs_to_struct_decl};

/// A render block, allowing for combining blocks in order to create more complex rendering
/// computations.
#[derive(Clone, Debug)]
struct Block {
  /// Unique identifier of the block. Used to get mangled names for structures and functions.
  id: String,
  inputs: Vec<Input>,
  outputs: Vec<Output>,
  code: Vec<ExternalDeclaration>
}

impl Block {
  /// Create a new block out of inputs, outputs and GLSL code.
  pub fn new<S, I, O, C>(id: S, inputs: I, outputs: O, code: C) -> Self
  where S: Into<String>,
        I: IntoIterator<Item = Input>,
        O: IntoIterator<Item = Output>,
        C: IntoIterator<Item = ExternalDeclaration> {
    Block {
      id: id.into(),
      inputs: inputs.into_iter().collect(),
      outputs: outputs.into_iter().collect(),
      code: code.into_iter().collect()
    }
  }

  /// Turn a block into its GLSL header representation.
  ///
  /// The GLSL header contains the inputs and the outputs struct definitions.
  fn to_glsl_header(&self) -> Vec<ExternalDeclaration> {
    // get the input and output GLSL representations
    let input_struct = inputs_to_struct_decl("In", &self.inputs);
    let output_struct = outputs_to_struct_decl("Out", &self.outputs);

    once(input_struct).chain(once(output_struct)).flatten().collect()
  }

  /// Turn a block into its GLSL full representation.
  ///
  /// The full GLSL representation includes:
  ///
  ///   - All intermediates structures, constants and functions.
  ///   - The *input* and *output* structures, renamed to:
  ///     - `In` becomes `In_<blockid>`. For instance, the block with ID `"Blur"` has its input type
  ///        going from `In` to `In_Blur`.
  ///     - `Out` becomes `Out_<blockid>`. For instance, the block with ID `"Blur"` has its output
  ///        type going from `Out` to `Out_Blur`.
  ///     - The function named `call` is searched for in the *code* part of the block and renamed
  ///       `call_<blockid>`. For instance, the block with ID `"Blur"` has its `call` function
  ///       renamed `call_Blur`.
  ///   - All references to `In`, `Out` and `call` are replaced with the appropriate new name.
  fn to_glsl(&self) -> Vec<ExternalDeclaration> {
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use glsl_quasiquote::glsl;

  use crate::render::types::*;
  use super::*;

  #[test]
  fn block_to_glsl_header() {
    use crate::render::input::Input;
    use crate::render::output::Output;

    let inputs = vec![Input::new::<Float, _>("time"), Input::new::<RGBAU, _>("bias")];
    let outputs = vec![Output::new::<RGBF, _>("color")];
    let code = vec![];
    let block = Block::new("simple", inputs, outputs, code);
    let expected = glsl!{
      struct In {
        float time;
        uvec4 bias;
      };

      struct Out {
        vec3 color;
      };
    };

    assert_eq!(block.to_glsl_header(), expected);
  }
}
