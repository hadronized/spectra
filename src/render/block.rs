//! Render combinatory blocks.
//!
//! Render blocks are logical rendering units that have inputs and outputs. Inputs can represent
//! vertex attributes, user-specified values, built-ins or previous blocks’ outputs.

use glsl::syntax::{Identifier, TranslationUnit, TypeName};
use glsl::visitor::{Host, Visit, Visitor};
use std::iter::once;

use crate::render::input::{Input, inputs_to_struct_decl};
use crate::render::output::{Output, outputs_to_struct_decl};

/// A render block, allowing for combining blocks in order to create more complex rendering
/// computations.
#[derive(Clone, Debug)]
pub struct Block {
  /// Unique identifier of the block. Used to get mangled names for structures and functions.
  id: String,
  /// List of inputs that compose the public interface of the block.
  inputs: Vec<Input>,
  /// List of outputs that compose the public interface of the block.
  outputs: Vec<Output>,
  /// Actual transformation code of the block.
  code: TranslationUnit
}

impl Block {
  /// Create a new block out of inputs, outputs and GLSL code.
  pub fn new<S, I, O, C>(id: S, inputs: I, outputs: O, code: C) -> Self
  where S: Into<String>,
        I: IntoIterator<Item = Input>,
        O: IntoIterator<Item = Output>,
        C: Into<TranslationUnit> {
    Block {
      id: id.into(),
      inputs: inputs.into_iter().collect(),
      outputs: outputs.into_iter().collect(),
      code: code.into()
    }
  }

  /// Turn a block into its GLSL header representation.
  ///
  /// The GLSL header contains the inputs and the outputs struct definitions.
  fn to_glsl_header(&self) -> Option<TranslationUnit> {
    // get the input and output GLSL representations
    let input_struct = inputs_to_struct_decl("In", &self.inputs);
    let output_struct = outputs_to_struct_decl("Out", &self.outputs);

    TranslationUnit::from_iter(once(input_struct).chain(once(output_struct)).flatten())
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
  fn to_glsl(&self) -> Option<TranslationUnit> {
    // generate the header
    let mut ast = self.to_glsl_header()?;

    // append the actual code
    ast.extend((&self.code).into_iter().cloned());

    // transform the AST to mangle In, Out and call()
    let mut mangler = Mangler::new(self.id.as_str());
    ast.visit(&mut mangler);

    Some(ast)
  }
}

/// AST visitor that mangles type names and function calls.
///
/// This works but appending a unique ID to names to replace. Replacements occur for:
///
///   - All intermediates structures, constants and functions, that reference `In`, `Out` or `call`.
///   - The *input* and *output* structures, renamed as:
///     - `In` becomes `In_<blockid>`. For instance, the block with ID `"Blur"` has its input type
///        going from `In` to `In_Blur`.
///     - `Out` becomes `Out_<blockid>`. For instance, the block with ID `"Blur"` has its output
///        type going from `Out` to `Out_Blur`.
///     - The function named `call` is searched for in the *code* part of the block and renamed
///       `call_<blockid>`. For instance, the block with ID `"Blur"` has its `call` function
///       renamed `call_Blur`.
///   - All references to `In`, `Out` and `call` are replaced with the appropriate new name.
struct Mangler<'a> {
  id: &'a str
}

impl<'a> Mangler<'a> {
  fn new(id: &'a str) -> Self {
    Mangler { id }
  }
}

impl<'a> Visitor for Mangler<'a> {
  fn visit_type_name(&mut self, type_name: &mut TypeName) -> Visit {
    match type_name.as_str() {
      "In" => *type_name = TypeName::new(format!("In_{}", self.id)).unwrap(),
      "Out" => *type_name = TypeName::new(format!("Out_{}", self.id)).unwrap(),
      _ => ()
    }

    Visit::Parent
  }

  fn visit_identifier(&mut self, identifier: &mut Identifier) -> Visit {
    match identifier.as_str() {
      "In" => *identifier = Identifier::new(format!("In_{}", self.id)).unwrap(),
      "Out" => *identifier = Identifier::new(format!("Out_{}", self.id)).unwrap(),
      "call" => *identifier = Identifier::new(format!("call_{}", self.id)).unwrap(),
      _ => ()
    }

    Visit::Parent
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
    let code = glsl!{ void main() {} };
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

    assert_eq!(block.to_glsl_header(), Some(expected));
  }

  #[test]
  fn block_to_glsl() {
    use crate::render::input::Input;
    use crate::render::output::Output;

    let inputs = vec![Input::new::<Float, _>("time"), Input::new::<RGBAF, _>("bias")];
    let outputs = vec![Output::new::<RGBF, _>("color")];

    let code = glsl!{
      Out call(In x) {
        return Out(x.bias.xyz * x.time);
      }
    };

    let block = Block::new("simple", inputs, outputs, code);

    let expected = glsl!{
      struct In_simple {
        float time;
        vec4 bias;
      };

      struct Out_simple {
        vec3 color;
      };

      Out_simple call_simple(In_simple x) {
        return Out_simple(x.bias.xyz * x.time);
      }
    };

    assert_eq!(block.to_glsl(), Some(expected));
  }
}
