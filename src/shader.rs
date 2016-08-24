use luminance::{FragmentShader, GeometryShader, StageError, ShaderTypeable,
                TessellationControlShader, TessellationEvaluationShader, VertexShader};
use luminance_gl::gl33::{ProgramProxy, Stage};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub use luminance::{ProgramError, UniformUpdate};
use luminance::shader::stage;
pub use luminance_gl::gl33::{Program, Uniform};

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}

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

pub fn read_stage<T, P>(path: P) -> Result<Stage<T>, StageError> where T: ShaderTypeable, P: AsRef<Path> {
  let path = path.as_ref().to_str().unwrap();

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

/// A `Program` wrapped by **ion**.
///
/// That wrapper is used to enable hot-reloading of shader programs.
#[cfg(feature = "hot-shader")]
pub struct WrappedProgram<'a, T> {
  receiver: mpsc::Receiver<(PathBuf, stage::Type)>,
  program: Program<T>,
  get_uni: Box<Fn(ProgramProxy) -> Result<T, ProgramError> + 'a>
}
#[cfg(not(feature = "hot-shader"))]
pub type WrappedProgram<'a, T> = Program<'a, T>;

/// A `WrappedProgram` manager.
pub struct WrappedProgramBuilder {

}

impl<'a, T> WrappedProgram<'a, T> {
  pub fn new<GetUni, P>(tess_path: Option<(P, P)>, vs_path: P, gs_path: Option<P>, fs_path: P, get_uni: GetUni) -> Result<Self, ProgramError>
      where GetUni: 'a + Fn(ProgramProxy) -> Result<T, ProgramError> + Clone,
            P: AsRef<Path> {

    // load vertex and fragment shaders first
    let vs = try!(read_stage(vs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
    let fs = try!(read_stage(fs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));

    let program = try!(match (tess_path, gs_path) {
      (None, None) => { // no tessellation nor geometry
        Program::new(None, &vs, None, &fs, get_uni.clone())
      },
      (Some((tcs_path, tes_path)), None) => { // tessellation without geometry
        let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        Program::new(Some((&tcs, &tes)), &vs, None, &fs, get_uni.clone())
      },
      (None, Some(gs_path)) => { // geometry without tessellation
        let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        Program::new(None, &vs, Some(&gs), &fs, get_uni.clone())
      },
      (Some((tcs_path, tes_path)), Some(gs_path)) => { // tessellation and geometry
        let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
        Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, get_uni.clone())
      }
    });

    let (sx, rx) = mpsc::channel();

    // TODO
    // start the watcher job

    let wrapped = WrappedProgram {
      receiver: rx,
      program: program,
      get_uni: Box::new(get_uni)
    };

    Ok(wrapped)
  }
}
