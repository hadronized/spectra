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
use std::io;
use std::ops::Deref;

use render::shader::cheddar::syntax::GLSLConversionError;
use render::shader::module::Module;
use sys::res::{Key, Load, Loaded, LogicalKey, Storage, StoreErrorOr};
use sys::res::helpers::{TyDesc, load_with};

/// Errors that can be risen by a shader.
#[derive(Debug)]
pub enum ShaderError {
  ModuleError(StoreErrorOr<Module>),
  GLSLConversionError(GLSLConversionError),
  ProgramError(ProgramError),
  KeyError(io::Error)
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
      ShaderError::ProgramError(_) => "program error",
      ShaderError::KeyError(_) => "key error"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      ShaderError::ModuleError(ref err) => Some(err),
      ShaderError::GLSLConversionError(ref err) => Some(err),
      ShaderError::ProgramError(ref err) => Some(err),
      ShaderError::KeyError(ref err) => Some(err)
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

impl<In, Out, Uni> TyDesc for Program<In, Out, Uni> {
  const TY_DESC: &'static str = "program";
}

impl<In, Out, Uni> Load for Program<In, Out, Uni>
    where In: 'static + Vertex,
          Out: 'static,
          Uni: 'static + UniformInterface {
  type Key = LogicalKey;

  type Error = ShaderError;

  fn load(key: Self::Key, storage: &mut Storage) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_str().as_ref();

    load_with::<Self, _, _>(path, move || {
      let module_key = Key::<Module>::path(path.to_owned()).map_err(|e| ShaderError::KeyError(e))?;
      let module = storage.get(&module_key).map_err(ShaderError::ModuleError)?;
      let (transitive, keys) = module.borrow().substitute_imports(&module_key, storage).map_err(|e| ShaderError::ModuleError(StoreErrorOr::ResError(e)))?;

      match transitive.to_glsl_setup() {
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

              let res = Program(program);
              let dep_keys = keys.into_iter().map(|key| key.into()).collect();

              Ok(Loaded::with_deps(res, dep_keys))
            }
          }
        }
      }
    })
  }

  impl_reload_passthrough!();
}

fn annotate_shader(s: &str) {
  for (i, line) in s.lines().enumerate() {
    info!("{:3}: {}", i + 1, line);
  }
}
