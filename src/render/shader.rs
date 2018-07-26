//! Shader programs.
//!
//! A shader program is a piece of code that runs on GPU, consuming vertices, primitives, geometry
//! patches and fragments and output into a framebuffer colors and depth information.
//!
//! Shaders are abstracted by *shader modules* – see the `render::shader::module` module for further
//! details. You are not supposed to write *shader programs* directly, but instead, you write one or
//! several *shader modules* and are automatically handed a *shader program* based on the analysis
//! of the *shader modules* you wrote and how you combined them.

use cheddar::{GLSLConversionError, Module};
use luminance::shader::program::Program as LProgram;
pub use luminance::shader::program::{ProgramError, Uniform, Uniformable, UniformBuilder,
                                     UniformInterface, UniformWarning};
use luminance::vertex::Vertex;
use std::error::Error;
use std::fmt;
use std::ops::Deref;
use std::path::Path;

use sys::res::{DepKey, FSKey, Key, Load, Loaded, LogicalKey, Storage, StoreErrorOr};
use sys::res::helpers::{TyDesc, load_with};

/// Key used to load shader programs.
///
/// It takes a module path as in `"foo.bar.zoo"`, the exact same way you do inside of your shader
/// source.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProgramKey(LogicalKey);

impl ProgramKey {
  pub fn new(path: &str) -> Self {
    ProgramKey(LogicalKey::new(path.replace('.', "/") + ".chdr"))
  }
}

impl Key for ProgramKey {
  fn prepare_key(self, root: &Path) -> Self {
    ProgramKey(self.0.prepare_key(root))
  }
}

impl From<ProgramKey> for DepKey {
  fn from(pkey: ProgramKey) -> Self {
    pkey.0.into()
  }
}

/// Errors that can be risen by a shader.
pub enum ShaderError<C> {
  ModuleError(StoreErrorOr<Module, C>),
  GLSLConversionError(GLSLConversionError),
  ProgramError(ProgramError),
}

impl<C> fmt::Debug for ShaderError<C> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ShaderError::ModuleError(ref e) => f.debug_tuple("ModuleError").field(e).finish(),
      ShaderError::GLSLConversionError(ref e) => f.debug_tuple("GLSLConversionError").field(e).finish(),
      ShaderError::ProgramError(ref e) => f.debug_tuple("ProgramError").field(e).finish(),
    }
  }
}

impl<C> fmt::Display for ShaderError<C> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    <Self as fmt::Debug>::fmt(self, f)
  }
}

impl<C> Error for ShaderError<C> {
  fn description(&self) -> &str {
    match *self {
      ShaderError::ModuleError(_) => "module error",
      ShaderError::GLSLConversionError(_) => "GLSL conversion error",
      ShaderError::ProgramError(_) => "program error",
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      ShaderError::ModuleError(ref err) => Some(err),
      ShaderError::GLSLConversionError(ref err) => Some(err),
      ShaderError::ProgramError(ref err) => Some(err),
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

impl<C, In, Out, Uni> Load<C> for Program<In, Out, Uni>
    where C: 'static,
          In: 'static + Vertex,
          Out: 'static,
          Uni: 'static + UniformInterface {
  type Key = ProgramKey;

  type Error = ShaderError<C>;

  fn load(key: Self::Key, storage: &mut Storage<C>, ctx: &mut C) -> Result<Loaded<Self>, Self::Error> {
    let path = key.0.as_str().as_ref();

    load_with::<Self, _, _, _>(path, move || {
      let module_key = FSKey::new(path);
      let module = storage.get(&module_key, ctx).map_err(ShaderError::ModuleError)?;
      let (transitive, keys) = module.borrow().substitute_imports(&module_key, storage, ctx)
                                     .map_err(|e| ShaderError::ModuleError(StoreErrorOr::ResError(e)))?;

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

  impl_reload_passthrough!(C);
}

fn annotate_shader(s: &str) {
  for (i, line) in s.lines().enumerate() {
    info!("{:3}: {}", i + 1, line);
  }
}
