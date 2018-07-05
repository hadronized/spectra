//! Current mode the server is in.

use spectra::render::shader::program::ProgramKey;

pub enum Mode {
  /// Nothing interesting.
  Empty,

  /// The server is in a shader toy like mode.
  ShaderToy(ProgramKey)
}

impl Default for Mode {
  fn default() -> Self {
    Mode::Empty
  }
}
