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
pub struct Program<In, Out, Uni> {
  program: LProgram<In, Out, Uni>
}

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = LProgram<In, Out, Uni>;

  fn deref(&self) -> &Self::Target {
    &self.program
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
  fn load<K>(key: &K, _: &mut Store) -> Result<LoadResult<Self>, LoadError> where K: StoreKey<Target = Self> {
    // load as a module first


    match File::open(&path) {
      Ok(fh) => {
        let buffered = BufReader::new(fh);
        let mut tcs_src = String::new();
        let mut tes_src = String::new();
        let mut vs_src = String::new();
        let mut gs_src = String::new();
        let mut fs_src = String::new();
        let mut current_stage: Option<CurrentStage> = None;

        for (line_nb, line) in buffered.lines().enumerate() {
          let line_nb = line_nb + 1;
          let line = line.unwrap();
          let trimmed = line.trim();

          if trimmed.starts_with("#vs") {
            if !vs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #vs sections", line_nb)));
            }

            info!("  found a vertex shader");

            current_stage = Some(CurrentStage::VS);
            continue;
          } else if trimmed.starts_with("#fs") {
            if !fs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #fs sections", line_nb)));
            }

            info!("  found a fragment shader");

            current_stage = Some(CurrentStage::FS);
            continue;
          } else if trimmed.starts_with("#gs") {
            if !gs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #gs sections", line_nb)));
            }

            info!("  found a geometry shader");

            current_stage = Some(CurrentStage::GS);
            continue;
          } else if trimmed.starts_with("#tcs") {
            if !tcs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #tcs sections", line_nb)));
            }

            info!("  found a tessellation control shader");

            current_stage = Some(CurrentStage::TCS);
            continue;
          } else if trimmed.starts_with("#tes") {
            if !tes_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #tes sections", line_nb)));
            }

            info!("  found a tessellation evaluation shader");

            current_stage = Some(CurrentStage::TES);
            continue;
          } else if current_stage.is_none() && !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("\n") {
            return Err(LoadError::ParseFailed(format!("(line {}) not in a shader stage nor a comment", line_nb)));
          }

          match current_stage {
            Some(CurrentStage::VS) => {
              annotate_line_src(&mut vs_src, trimmed, line_nb);
            },
            Some(CurrentStage::FS) => {
              annotate_line_src(&mut fs_src, trimmed, line_nb);
            },
            Some(CurrentStage::GS) => {
              annotate_line_src(&mut gs_src, trimmed, line_nb);
            },
            Some(CurrentStage::TCS) => {
              annotate_line_src(&mut tcs_src, trimmed, line_nb);
            },
            Some(CurrentStage::TES) => {
              annotate_line_src(&mut tes_src, trimmed, line_nb);
            },
            None => {}
          }
        }

        let (program, warnings) = new_program(&tcs_src, &tes_src, &vs_src, &gs_src, &fs_src)
          .map_err(|e| LoadError::ConversionFailed(format!("{:#?}", e)))?;

        // check for semantic errors
        for warning in warnings {
          warn!("uniform warning: {:?}", warning);
        }

        Ok(
          (Program {
            program: program
          }).into()
        )
      },
      Err(_) => Err(LoadError::FileNotFound(path.into()))
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

// FIXME: test only
pub fn from_spsl(key: &ModuleKey, store: &mut Store) {
  if let Some(module) = store.get(key) {
    let (gathered, _) = module.borrow().gather(store, key).unwrap();
    let glsl = gathered.to_glsl_setup();

    match glsl {
      Ok(s) => {
        println!("success");
        println!("vertex shader:\n{}", s.vs);
        println!("\nfragment shader shader:\n{}", s.fs);
      }
      Err(e) => {
        println!("damn: {:?}", e)
      }
    }
  }
}
