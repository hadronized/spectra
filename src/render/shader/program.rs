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

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}

/// Create a new luminance::Program from a set of shader strings.
fn new_program<In, Out, Uni>(tcs_src: &str,
                             tes_src: &str,
                             vs_src: &str,
                             gs_src: &str,
                             fs_src: &str)
                             -> Result<(LProgram<In, Out, Uni>, Vec<UniformWarning>), ProgramError>
    where In: Vertex,
          Uni: UniformInterface {
  let stages = compile_stages(tcs_src, tes_src, vs_src, gs_src, fs_src);

  match stages {
    Ok((tess, vs, gs, fs)) => {
      if let Some((tcs, tes)) = tess {
        if let Some(gs) = gs {
          LProgram::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs)
        } else {
          LProgram::new(Some((&tcs, &tes)), &vs, None, &fs)
        }
      } else if let Some(gs) = gs {
        LProgram::new(None, &vs, Some(&gs), &fs)
      } else {
        LProgram::new(None, &vs, None, &fs)
      }
    },
    Err(stage_error) => {
      Err(ProgramError::LinkFailed(format!("{:?}", stage_error)))
    }
  }
}

/// Take raw shader sources and turn them into stages.
fn compile_stages(tcs_src: &str, tes_src: &str, vs_src: &str, gs_src: &str, fs_src: &str) -> Result<(Option<(Stage, Stage)>, Stage, Option<Stage>, Stage), StageError> {
  let tess = if !tcs_src.is_empty() && !tes_src.is_empty() {
    Some((Stage::new(Type::TessellationControlShader, tcs_src)?,
          Stage::new(Type::TessellationEvaluationShader, tes_src)?))
  } else {
    None
  };

  let vs = Stage::new(Type::VertexShader, vs_src)?;
  let gs = if !gs_src.is_empty() { Some(Stage::new(Type::GeometryShader, gs_src)?) } else { None };
  let fs = Stage::new(Type::FragmentShader, fs_src)?;

  Ok((tess, vs, gs, fs))
}


/// Shader program.
pub struct Program<In, Out, Uni> {
  program: LProgram<In, Out, Uni>
}

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = LProgram<In, Out, Uni>;

  fn deref(&self) -> &Self::Target {
    &self.program
  }
}

// Current stage dispatch.
enum CurrentStage {
  VS,
  FS,
  GS,
  TCS,
  TES
}

struct ShaderSources {
  tcs_src: String,
  tes_src: String,
  vs_src: String,
  gs_src: String,
  fs_src: String
}

// Annotate a line with its original line number.
fn annotate_line_src(src: &mut String, line: &str, line_nb: usize) {
  *src += &format!("#line {}\n{}\n", line_nb, line);
}

// Split a single source into several shader sources.
fn split_shader_stages_sources<R>(buffered: R) -> Result<ShaderSources, LoadError> where R: BufRead {
  let mut current_stage: Option<CurrentStage> = None;
  let mut tcs_src = String::new();
  let mut tes_src = String::new();
  let mut vs_src = String::new();
  let mut gs_src = String::new();
  let mut fs_src = String::new();

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
      Some(CurrentStage::TCS) => {
        annotate_line_src(&mut tcs_src, trimmed, line_nb);
      },
      Some(CurrentStage::TES) => {
        annotate_line_src(&mut tes_src, trimmed, line_nb);
      },
      Some(CurrentStage::VS) => {
        annotate_line_src(&mut vs_src, trimmed, line_nb);
      },
      Some(CurrentStage::GS) => {
        annotate_line_src(&mut gs_src, trimmed, line_nb);
      },
      Some(CurrentStage::FS) => {
        annotate_line_src(&mut fs_src, trimmed, line_nb);
      },
      None => {}
    }
  }

  Ok(ShaderSources {
    tcs_src: tcs_src,
    tes_src: tes_src,
    vs_src: vs_src,
    gs_src: gs_src,
    fs_src: fs_src
  })
}

impl<In, Out, Uni> Program<In, Out, Uni> where In: Vertex, Uni: UniformInterface {
  /// Create a `Program` from a `BufRead`.
  pub fn from_bufread<R>(buffered: R) -> Result<Self, LoadError> where R: BufRead {
    let sources = split_shader_stages_sources(buffered)?;
    let (program, warnings) = new_program(&sources.tcs_src,
                                          &sources.tes_src,
                                          &sources.vs_src,
                                          &sources.gs_src,
                                          &sources.fs_src)
      .map_err(|e| LoadError::ConversionFailed(format!("{:#?}", e)))?;

    // check for semantic errors
    for warning in warnings {
      warn!("uniform warning: {:?}", warning);
    }

    Ok(Program {
      program: program
    })
  }

  /// Create a `Program` from a string – you can for instance use `str` or `String`.
  pub fn from_str<'a, S>(s: S) -> Result<Self, LoadError> where S: Into<&'a str> {
    Self::from_bufread(s.into().as_bytes())
  }
}

#[derive(Eq, PartialEq)]
pub struct ProgramKey<In, Out, Uni> {
  pub key: String,
  _in: PhantomData<*const In>,
  _out: PhantomData<*const Out>,
  _uni: PhantomData<*const Uni>
}

impl<In, Out, Uni> ProgramKey<In, Out, Uni> {
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
    let path = key.key_to_path();

    enum CurrentStage {
      VS,
      FS,
      GS,
      TCS,
      TES
    }

    fn annotate_line_src(src: &mut String, line: &str, line_nb: usize) {
      *src += &format!("#line {}\n{}\n", line_nb, line);
    }

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
        println!("success: {:?}", s);
      }
      Err(e) => {
        println!("damn: {:?}", e)
      }
    }
  }
}
