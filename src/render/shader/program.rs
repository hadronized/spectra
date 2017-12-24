//! Shader programs.
//!
//! A shader program is a piece of code that runs on GPU, consuming vertices, primitives, geometry
//! patches and fragments and output into a framebuffer colors and depth information.
//!
//! Shaders are abstracted by *shader modules* – see the `render::shader::module` module for further
//! details. You are not supposed to write *shader programs* directly, but instead, you write one or
//! several *shader modules* and are automatically handed a *shader program* based on the analysis
//! of the *shader modules* you wrote and how you combined them.
use luminance::shader::program::Program as LProgram;
pub use luminance::shader::program::{ProgramError, Uniform, Uniformable, UniformBuilder,
                                     UniformInterface, UniformWarning};
use luminance::vertex::Vertex;
use std::error::Error;
use std::fmt;
use std::ops::Deref;
use std::path::Path;

use render::shader::cheddar::syntax::GLSLConversionError;
use render::shader::module::{Module, ModuleError};
use sys::resource::{DebugRes, Key, Load, Loaded, Store, load_with};

/// Errors that can be risen by a shader.
#[derive(Debug)]
pub enum ShaderError {
  ModuleError(ModuleError),
  GLSLConversionError(GLSLConversionError),
  ProgramError(ProgramError)
}

impl fmt::Display for ShaderError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for ShaderError {
  fn description(&self) -> &str {
    match *self {
      ShaderError::ModuleError(_) => "module error",
      ShaderError::GLSLConversionError(_) => "GLSL conversion error",
      ShaderError::ProgramError(_) => "program error"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      ShaderError::ModuleError(ref err) => Some(err),
      ShaderError::GLSLConversionError(ref err) => Some(err),
      ShaderError::ProgramError(ref err) => Some(err)
    }
  }
}

/// Shader program.
///
/// This program must be used in a pipeline to take effect.
pub struct Program<In, Out, Uni>(LProgram<In, Out, Uni>);

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = LProgram<In, Out, Uni>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<In, Out, Uni> DebugRes for Program<In, Out, Uni> {
  const TYPE_DESC: &'static str = "program";
}

impl<In, Out, Uni> Load for Program<In, Out, Uni>
    where In: 'static + Vertex,
          Out: 'static,
          Uni: 'static + UniformInterface {
  type Error = ShaderError;

  fn from_fs<P>(path: P, store: &mut Store) -> Result<Loaded<Self>, Self::Error> where P: AsRef<Path> {
    let path = path.as_ref();

    load_with::<Self, _, _>(path, move || {
      let module_key = Key::<Module>::new(path.to_owned());
      let module = store.get(&module_key).map_err(ShaderError::ModuleError)?;

      let module_ = module.borrow();

      match module_.to_glsl_setup() {
        Err(err) => {
          Err(ShaderError::GLSLConversionError(err))
        }
        Ok(fold) => {
          deb!("vertex shader");
          annotate_shader(&fold.vs);

          if let Some(ref gs) = fold.gs {
            deb!("geometry shader");
            annotate_shader(gs);
          }

          deb!("fragment shader");
          annotate_shader(&fold.fs);

          match LProgram::from_strings(None, &fold.vs, fold.gs.as_ref().map(String::as_str), &fold.fs) {
            Err(err) => {
              Err(ShaderError::ProgramError(err))
            }
            Ok((program, warnings)) => {
              // print warnings in case there’s any
              for warning in &warnings {
                warn!("{:?}", warning);
              }

              Ok(Program(program).into()) // FIXME: deps
            }
          }
        }
      }
    })
  }
}

fn annotate_shader(s: &str) {
  for (i, line) in s.lines().enumerate() {
    info!("{:3}: {}", i + 1, line);
  }
}

pub trait UnwrapOrUnbound<T> {
  fn unwrap_or_unbound(self, builder: &UniformBuilder, warnings: &mut Vec<UniformWarning>) -> Uniform<T> where T: Uniformable;
}

impl<T> UnwrapOrUnbound<T> for Result<Uniform<T>, UniformWarning> {
  fn unwrap_or_unbound(self, builder: &UniformBuilder, warnings: &mut Vec<UniformWarning>) -> Uniform<T> where T: Uniformable {
    self.unwrap_or_else(|w| {
      warnings.push(w);
      builder.unbound()
    })
  }
}
