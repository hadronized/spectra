use luminance::{FragmentShader, GeometryShader, StageError, ShaderTypeable,
                TessellationControlShader, TessellationEvaluationShader, VertexShader};
use luminance_gl::gl33::{ProgramProxy, Stage};
use std::fs;
use std::io::Read;
use std::path::Path;

pub use luminance::{ProgramError, Uniformable, UniformUpdate};
pub use luminance::shader::program::UniformWarning;
pub use luminance_gl::gl33::{self, Uniform};
pub use luminance_gl::gl33::token::*;

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

pub fn new_program<GetUni, T>(tess_src: Option<(&str, &str)>, vs_src: &str, gs_src: Option<&str>, fs_src: &str, get_uni: &GetUni) -> Result<gl33::Program<T>, ProgramError> where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> {
  let stages = compile_stages(tess_src, vs_src, gs_src, fs_src);

  match stages {
    Ok((tess, vs, gs, fs)) => {
      if let Some((tcs, tes)) = tess {
        if let Some(gs) = gs {
          gl33::Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, get_uni)
        } else {
          gl33::Program::new(Some((&tcs, &tes)), &vs, None, &fs, get_uni)
        }
      } else if let Some(gs) = gs {
        gl33::Program::new(None, &vs, Some(&gs), &fs, get_uni)
      } else {
        gl33::Program::new(None, &vs, None, &fs, get_uni)
      }
    },
    Err(stage_error) => {
      Err(ProgramError::LinkFailed(format!("{:?}", stage_error)))
    }
  }
}

pub fn new_program_from_disk<P, GetUni, T>(tess_path: Option<(P, P)>, vs_path: P, gs_path: Option<P>, fs_path: P, get_uni: &GetUni) -> Result<gl33::Program<T>, ProgramError>
      where P: AsRef<Path>,
            GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> {
  // load vertex and fragment shaders first
  let vs = try!(read_stage(vs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
  let fs = try!(read_stage(fs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));

  match (tess_path, gs_path) {
    (None, None) => { // no tessellation nor geometry
      gl33::Program::new(None, &vs, None, &fs, get_uni)
    },
    (Some((tcs_path, tes_path)), None) => { // tessellation without geometry
      let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      gl33::Program::new(Some((&tcs, &tes)), &vs, None, &fs, get_uni)
    },
    (None, Some(gs_path)) => { // geometry without tessellation
      let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      gl33::Program::new(None, &vs, Some(&gs), &fs, get_uni)
    },
    (Some((tcs_path, tes_path)), Some(gs_path)) => { // tessellation and geometry
      let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      gl33::Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, get_uni)
    }
  }
}

pub fn read_stage<P, T>(path: P) -> Result<Stage<T>, StageError> where P: AsRef<Path>, T: ShaderTypeable {
  let path = path.as_ref();

  info!("loading {:?} stage: \x1b[35m{:?}", T::shader_type(), path);

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

#[cfg(feature = "hot-resource")]
mod hot {
  use super::*;

  use luminance_gl::gl33::ProgramProxy;
  use resource::ResourceManager;
  use std::ops::Deref;
  use std::path::{Path, PathBuf};
  use std::sync::mpsc;

  pub struct Program<T> {
    rx: mpsc::Receiver<()>,
    last_update_time: Option<f64>,
    program: gl33::Program<T>,
    get_uni: Box<Fn(ProgramProxy) -> Result<T, ProgramError>>,
    vs_path: PathBuf,
    fs_path: PathBuf,
    tess_path: Option<(PathBuf, PathBuf)>,
    gs_path: Option<PathBuf>
  }

  impl<T> Program<T> {
    pub fn load<GetUni, P>(manager: &mut ResourceManager, tess_path: Option<(P, P)>, vs_path: P, gs_path: Option<P>, fs_path: P, get_uni: GetUni) -> Result<Self, ProgramError>
        where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> + 'static,
              P: AsRef<Path> {
      let vs_path = vs_path.as_ref();
      let fs_path = fs_path.as_ref();
      let tess_path = tess_path.as_ref().map(|&(ref tcs, ref tes)| (tcs.as_ref(), tes.as_ref()));
      let gs_path = gs_path.as_ref().map(|gs| gs.as_ref());

      let program = try!(new_program_from_disk(tess_path, vs_path, gs_path, fs_path, &get_uni));

      let (sx, rx) = mpsc::channel();
      manager.monitor(vs_path, sx.clone());
      manager.monitor(fs_path, sx.clone());

      if let Some((tcs, tes)) = tess_path {
        manager.monitor(tcs, sx.clone());
        manager.monitor(tes, sx.clone());
      }

      if let Some(gs) = gs_path {
        manager.monitor(gs, sx.clone());
      }

      Ok(Program {
        rx: rx,
        last_update_time: None,
        program: program,
        get_uni: Box::new(get_uni),
        vs_path: vs_path.to_path_buf(),
        fs_path: fs_path.to_path_buf(),
        tess_path: tess_path.map(|(tcs, tes)| (tcs.to_path_buf(), tes.to_path_buf())),
        gs_path: gs_path.map(|gs| gs.to_path_buf()),
      })
    }

    fn reload(&mut self) {
      let vs = self.vs_path.as_path();
      let fs = self.fs_path.as_path();
      let tess = self.tess_path.as_ref().map(|&(ref tcs, ref tes)| (tcs.as_path(), tes.as_path()));
      let gs = self.gs_path.as_ref().map(|gs| gs.as_path());

      let program = new_program_from_disk(tess, vs, gs, fs, &self.get_uni.as_ref());

      match program {
        Ok(program) => {
          self.program = program;
          info!("reloaded shader program");
        },
        Err(e) => {
          err!("reloading program has failed: {:?}", e);
        }
      }
    }

    decl_sync_hot!();
  }

  impl<'a, T> Deref for Program<T> {
    type Target = gl33::Program<T>;

    fn deref(&self) -> &Self::Target {
      &self.program
    }
  }
}

#[cfg(not(feature = "hot-resource"))]
mod cold {
  use super::*;

  use luminance_gl::gl33::ProgramProxy;
  use resource::ResourceManager;
  use std::ops::Deref;
  use std::path::Path;

  pub struct Program<T>(gl33::Program<T>);

  impl<T> Program<T> {
    pub fn load<GetUni, P>(_: &mut ResourceManager, tess_path: Option<(P, P)>, vs_path: P, gs_path: Option<P>, fs_path: P, get_uni: GetUni) -> Result<Program<T>, ProgramError>
        where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> + 'static,
              P: AsRef<Path> {
      let vs_path = vs_path.as_ref();
      let fs_path = fs_path.as_ref();
      let tess_path = tess_path.as_ref().map(|&(ref tcs, ref tes)| (tcs.as_ref(), tes.as_ref()));
      let gs_path = gs_path.as_ref().map(|gs| gs.as_ref());

      let program = try!(new_program_from_disk(tess_path, vs_path, gs_path, fs_path, &get_uni));

      Ok(Program(program))
    }

    pub fn sync(&mut self) {}
  }

  impl<'a, T> Deref for Program<T> {
    type Target = gl33::Program<T>;

    fn deref(&self) -> &Self::Target {
      &self.0
    }
  }
}

#[cfg(feature = "hot-resource")]
pub use self::hot::*;
#[cfg(not(feature = "hot-resource"))]
pub use self::cold::*;

/// A helper function used to make uniforms optionable. If there’s a warning, it’s printed out.
pub fn opt_uni<T>(uni: (Uniform<T>, Option<UniformWarning>)) -> Uniform<T> where T: Uniformable<GL33> {
  if let Some(warning) = uni.1 {
    warn!("{:?}", warning);
  }

  uni.0
}
