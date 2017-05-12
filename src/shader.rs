use luminance::shader::program::Program as LProgram;
use luminance::shader::stage::{Stage, StageError, Type};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::path::Path;

pub use luminance::shader::program::{ProgramError, Uniform, Uniformable, UniformBuilder,
                                     UniformInterface, UniformWarning};
use luminance::vertex::Vertex;

use resource::{Load, LoadError, ResCache};

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}

pub fn new_program<In, Out, Uni>(tcs_src: &str,
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

// Take raw shader sources and turn them into stages.
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
pub struct Program<In, Out, Uni> {
  program: LProgram<In, Out, Uni>
}

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = LProgram<In, Out, Uni>;

  fn deref(&self) -> &Self::Target {
    &self.program
  }
}

impl<In, Out, Uni> Load for Program<In, Out, Uni> where In: Vertex, Uni: UniformInterface {
  type Args = ();

  const TY_STR: &'static str = "shaders";

  fn load<P>(path: P, _: &mut ResCache, _: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    info!("loading shader: {:?}", path);

    enum CurrentStage {
      VS,
      FS,
      GS,
      TCS,
      TES
    }

    fn add_line_to_src(src: &mut String, line: &str, line_nb: usize) {
      *src += &format!("#line {}\n{}\n", line_nb, line);
    }

    match File::open(path) {
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
              add_line_to_src(&mut vs_src, trimmed, line_nb);
            },
            Some(CurrentStage::FS) => {
              add_line_to_src(&mut fs_src, trimmed, line_nb);
            },
            Some(CurrentStage::GS) => {
              add_line_to_src(&mut gs_src, trimmed, line_nb);
            },
            Some(CurrentStage::TCS) => {
              add_line_to_src(&mut tcs_src, trimmed, line_nb);
            },
            Some(CurrentStage::TES) => {
              add_line_to_src(&mut tes_src, trimmed, line_nb);
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
          Program {
            program: program
          }
        )
      },
      Err(_) => Err(LoadError::FileNotFound(path.to_owned()))
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
