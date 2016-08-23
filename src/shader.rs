use luminance::{FragmentShader, GeometryShader, StageError, ShaderTypeable,
                TessellationControlShader, TessellationEvaluationShader, VertexShader};
use luminance_gl::gl33::{ProgramProxy, Stage};
use std::fs;
use std::io::Read;

pub use luminance::{ProgramError, UniformUpdate};
pub use luminance_gl::gl33::{Program, Uniform};

fn compile_stages(tess_src: Option<(&str, &str)>, vs_src: &str, gs_src: Option<&str>, fs_src: &str) -> Result<(Option<(Stage<TessellationControlShader>, Stage<TessellationEvaluationShader>)>, Stage<VertexShader>, Option<Stage<GeometryShader>>, Stage<FragmentShader>), StageError> {
  let tess = match tess_src {
    None => None,
    Some((tcs_src, tes_src)) => {
      let tcs = try!(Stage::new(tcs_src));
      let tes = try!(Stage::new(tes_src));
      Some((tcs, tes))
    }
  };
  let vs = try!(Stage::new(vs_src));
  let gs = match gs_src {
    None => None,
    Some(gs_src) => {
      let gs = try!(Stage::new(gs_src));
      Some(gs)
    }
  };
  let fs = try!(Stage::new(fs_src));

  Ok((tess, vs, gs, fs))
}

pub fn new_program<GetUni, T>(tess_src: Option<(&str, &str)>, vs_src: &str, gs_src: Option<&str>, fs_src: &str, get_uni: GetUni) -> Result<Program<T>, ProgramError> where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> {
  let stages = compile_stages(tess_src, vs_src, gs_src, fs_src);

  match stages {
    Ok((tess, vs, gs, fs)) => {
      if let Some((tcs, tes)) = tess {
        if let Some(gs) = gs {
          Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, get_uni)
        } else {
          Program::new(Some((&tcs, &tes)), &vs, None, &fs, get_uni)
        }
      } else {
        if let Some(gs) = gs {
          Program::new(None, &vs, Some(&gs), &fs, get_uni)
        } else {
          Program::new(None, &vs, None, &fs, get_uni)
        }
      }
    },
    Err(stage_error) => {
      Err(ProgramError::LinkFailed(format!("{:?}", stage_error)))
    }
  }
}

pub fn read_stage<T>(path: &str) -> Result<Stage<T>, StageError> where T: ShaderTypeable {
  info!("\tloading {:?} stage: \x1b[35m{}", T::shader_type(), path);

  let fh = fs::File::open(path);

  match fh {
    Err(e) => {
      Err(StageError::CompilationFailed(T::shader_type(), format!("{:?}", e)))
    },
    Ok(mut fh) => {
      let mut stage_src = String::new();
      let _ = fh.read_to_string(&mut stage_src);

      Stage::new(stage_src.chars().as_str())
    }
  }
}

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}
