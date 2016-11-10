use luminance::StageError;
use luminance::shader::stage;
use luminance_gl::gl33::Stage;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::path::Path;

pub use luminance::{ProgramError, Sem, Uniformable};
pub use luminance::shader::program::UniformWarning;
pub use luminance_gl::gl33::{self, Uniform};
pub use luminance_gl::gl33::token::GL33;

use resource::{Cache, Load, LoadError, Reload};

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}

pub fn new_program(tcs_src: &str, tes_src: &str, vs_src: &str, gs_src: &str, fs_src: &str, sem_map: &[Sem]) -> Result<(gl33::Program, Vec<UniformWarning>), ProgramError> {
  let stages = compile_stages(tcs_src, tes_src, vs_src, gs_src, fs_src);

  match stages {
    Ok((tess, vs, gs, fs)) => {
      if let Some((tcs, tes)) = tess {
        if let Some(gs) = gs {
          gl33::Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, sem_map)
        } else {
          gl33::Program::new(Some((&tcs, &tes)), &vs, None, &fs, sem_map)
        }
      } else if let Some(gs) = gs {
        gl33::Program::new(None, &vs, Some(&gs), &fs, sem_map)
      } else {
        gl33::Program::new(None, &vs, None, &fs, sem_map)
      }
    },
    Err(stage_error) => {
      Err(ProgramError::LinkFailed(format!("{:?}", stage_error)))
    }
  }
}

// Take raw shader sources and turn them into stages.
fn compile_stages(tcs_src: &str, tes_src: &str, vs_src: &str, gs_src: &str, fs_src: &str) -> Result<(Option<(Stage, Stage)>, Stage, Option<Stage>, Stage), StageError> {
  let tess = if !tcs_src.is_empty() && !tes_src.is_empty() {
    Some((Stage::new(stage::Type::TessellationControlShader, tcs_src)?,
          Stage::new(stage::Type::TessellationEvaluationShader, tes_src)?))
  } else {
    None
  };

  let vs = Stage::new(stage::Type::VertexShader, vs_src)?;
  let gs = if !gs_src.is_empty() { Some(Stage::new(stage::Type::GeometryShader, gs_src)?) } else { None };
  let fs = Stage::new(stage::Type::FragmentShader, fs_src)?;

  Ok((tess, vs, gs, fs))
}

/// Shader program.
///
/// If the program is retrieved from the cache, the path must point to a file containing all the
/// stages. A stage begins with a stage pragma indicating which kind of stage the following source
/// is. Here’s a list of all pragma for each kind of stage:
///
/// - `#tcs`: *tessellation control stage*
/// - `#tes`: *tessellation evaluation stage*
/// - `#vs`: *vertex stage*
/// - `#gs`: *geometry stage*
/// - `#fs`: *fragment stage*
///
/// A stage starts at such a pragma listed above and ends at the next, different pragma. You cannot
/// use twice the same pragma in a file.
///
/// At the top of the file, if you don’t put a pragma, you can use `//` to add comments, or die.
pub struct Program {
  program: gl33::Program,
  sem_map: Vec<Sem>
}

impl Deref for Program {
  type Target = gl33::Program;

  fn deref(&self) -> &Self::Target {
    &self.program
  }
}

impl Load for Program {
  type Args = Vec<Sem>;

  fn load<'a, P>(path: P, _: &mut Cache<'a>, args: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path> {
    enum CurrentStage {
      VS,
      FS,
      GS,
      TCS,
      TES
    }

    fn add_line_to_src(src: &mut String, line: &String, line_nb: usize) {
      if src.is_empty() {
        *src += &format!("#line {}\n{}\n", line_nb, line);
      }
    }

    match File::open(path.as_ref()) {
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

            current_stage = Some(CurrentStage::VS);
          } else if trimmed.starts_with("#fs") {
            if !fs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #fs sections", line_nb)));
            }

            current_stage = Some(CurrentStage::FS);
          } else if trimmed.starts_with("#gs") {
            if !gs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #gs sections", line_nb)));
            }

            current_stage = Some(CurrentStage::GS);
          } else if trimmed.starts_with("#tcs") {
            if !tcs_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #tcs sections", line_nb)));
            }

            current_stage = Some(CurrentStage::TCS);
          } else if trimmed.starts_with("#tes") {
            if !tes_src.is_empty() {
              return Err(LoadError::ParseFailed(format!("(line {}) several #tes sections", line_nb)));
            }

            current_stage = Some(CurrentStage::TES);
          } else if current_stage.is_none() && !trimmed.starts_with("//") {
            return Err(LoadError::ParseFailed(format!("(line {}) not in a shader stage nor a comment", line_nb)));
          }

          match current_stage {
            Some(CurrentStage::VS) => {
              add_line_to_src(&mut vs_src, &line, line_nb);
            },
            Some(CurrentStage::FS) => {
              add_line_to_src(&mut fs_src, &line, line_nb);
            },
            Some(CurrentStage::GS) => {
              add_line_to_src(&mut gs_src, &line, line_nb);
            },
            Some(CurrentStage::TCS) => {
              add_line_to_src(&mut tcs_src, &line, line_nb);
            },
            Some(CurrentStage::TES) => {
              add_line_to_src(&mut tes_src, &line, line_nb);
            },
            None => {}
          }
        }

        let (program, warnings) = new_program(&tcs_src, &tes_src, &vs_src, &gs_src, &fs_src, &args)
          .map_err(|e| LoadError::ConversionFailed(format!("{:?}", e)))?;

        // check for semantic errors
        for warning in warnings {
          warn!("uniform warning: {:?}", warning);
        }

        Ok(
          Program {
            program: program,
            sem_map: args
          }
        )
      },
      Err(e) => {
        Err(LoadError::FileNotFound(path.as_ref().to_owned(), format!("{:?}", e)))
      }
    }
  }
}

impl Reload for Program {
  fn reload_args(&self) -> Self::Args {
    self.sem_map.clone()
  }
}

/// A helper function used to make uniforms optionable. If there’s a warning, it’s printed out.
pub fn opt_uni<T>(uni: (Uniform<T>, Option<UniformWarning>)) -> Uniform<T> where T: Uniformable<GL33> {
  if let Some(warning) = uni.1 {
    warn!("{:?}", warning);
  }

  uni.0
}
