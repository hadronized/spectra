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
use luminance::shader::stage::{Stage, StageError, Type};
use luminance::vertex::Vertex;
use std::fmt;
use std::fs::File;
use std::hash;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::PathBuf;

use render::shader::module::ModuleKey;
use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};

/// Errors that can be risen by a shader.
#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
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

/// Key for loading `Program`s.
#[derive(Eq, PartialEq)]
pub struct ProgramKey<In, Out, Uni> {
  pub key: String,
  _in: PhantomData<*const In>,
  _out: PhantomData<*const Out>,
  _uni: PhantomData<*const Uni>
}

impl<In, Out, Uni> ProgramKey<In, Out, Uni> {
  /// Create a new `Program` key.
  ///
  /// A `ProgramKey` must reference a module. See the documentation of `ModuleKey` for further
  /// details.
  pub fn new(key: &str) -> Self {
    ProgramKey {
      key: key.to_owned(),
      _in: PhantomData,
      _out: PhantomData,
      _uni: PhantomData
    }
  }
}

impl<In, Out, Uni> Clone for ProgramKey<In, Out, Uni> {
  fn clone(&self) -> Self {
    ProgramKey {
      key: self.key.clone(),
      ..*self
    }
  }
}

impl<In, Out, Uni> fmt::Debug for ProgramKey<In, Out, Uni> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    self.key.fmt(f)
  }
}

impl<In, Out, Uni> hash::Hash for ProgramKey<In, Out, Uni> {
  fn hash<H>(&self, hasher: &mut H) where H: hash::Hasher {
    self.key.hash(hasher)
  }
}

impl<'a, In, Out, Uni> From<&'a str> for ProgramKey<In, Out, Uni> {
  fn from(key: &str) -> Self {
    ProgramKey::new(key)
  }
}

impl<In, Out, Uni> CacheKey for ProgramKey<In, Out, Uni>
    where In: 'static,
          Out: 'static,
          Uni: 'static {
  type Target = Program<In, Out, Uni>;
}

impl<In, Out, Uni> StoreKey for ProgramKey<In, Out, Uni>
    where In: 'static,
          Out: 'static,
          Uni: 'static {
  fn key_to_path(&self) -> PathBuf {
    self.key.clone().into()
  }
}

impl<In, Out, Uni> Load for Program<In, Out, Uni>
    where In: 'static + Vertex,
          Out: 'static,
          Uni: 'static + UniformInterface {
  type Key = ProgramKey<In, Out, Uni>;

  fn load(key: &Self::Key, store: &mut Store) -> Result<LoadResult<Self>, LoadError> {
    let module_key = ModuleKey::new(&key.key);
    let module = store.get(&module_key).ok_or(LoadError::ConversionFailed("cannot get program".to_owned()))?;

    let module_ = module.borrow();
    match module_.to_glsl_setup() {
      Err(err) => {
        err!("{:?}", err);
        Err(LoadError::ConversionFailed("cannot generate GLSL".to_owned()))
      }
      Ok(fold) => {
        match LProgram::from_strings(None, &fold.vs, None, &fold.fs) {
          Err(err) => {
            err!("{:?}", err);
            Err(LoadError::ConversionFailed("damn".to_owned()))
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
