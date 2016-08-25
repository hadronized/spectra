use luminance::{FragmentShader, GeometryShader, StageError, ShaderTypeable,
                TessellationControlShader, TessellationEvaluationShader, VertexShader};
use luminance_gl::gl33::{ProgramProxy, Stage};
use notify::{self, RecommendedWatcher, Watcher};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

pub use luminance::{ProgramError, UniformUpdate};
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

pub fn new_program<GetUni, T>(tess_src: Option<(&str, &str)>, vs_src: &str, gs_src: Option<&str>, fs_src: &str, get_uni: &GetUni) -> Result<Program<T>, ProgramError> where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> {
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

pub fn new_program_from_disk<GetUni, T>(tess_path: Option<(PathBuf, PathBuf)>, vs_path: PathBuf, gs_path: Option<PathBuf>, fs_path: PathBuf, get_uni: &GetUni) -> Result<Program<T>, ProgramError>
      where GetUni: Fn(ProgramProxy) -> Result<T, ProgramError> {
  // load vertex and fragment shaders first
  let vs = try!(read_stage(vs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
  let fs = try!(read_stage(fs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));

  match (tess_path, gs_path) {
    (None, None) => { // no tessellation nor geometry
      Program::new(None, &vs, None, &fs, get_uni)
    },
    (Some((tcs_path, tes_path)), None) => { // tessellation without geometry
      let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      Program::new(Some((&tcs, &tes)), &vs, None, &fs, get_uni)
    },
    (None, Some(gs_path)) => { // geometry without tessellation
      let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      Program::new(None, &vs, Some(&gs), &fs, get_uni)
    },
    (Some((tcs_path, tes_path)), Some(gs_path)) => { // tessellation and geometry
      let tcs = try!(read_stage(tcs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let tes = try!(read_stage(tes_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      let gs = try!(read_stage(gs_path).map_err(|e| ProgramError::LinkFailed(format!("{:?}", e))));
      Program::new(Some((&tcs, &tes)), &vs, Some(&gs), &fs, get_uni)
    }
  }
}

pub fn read_stage<T>(path: PathBuf) -> Result<Stage<T>, StageError> where T: ShaderTypeable {
  info!("\tloading {:?} stage: \x1b[35m{:?}", T::shader_type(), path);

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

/// Shader builder.
pub struct ProgramBuilder {
  watcher: RecommendedWatcher,
  receivers: Arc<Mutex<BTreeMap<PathBuf, mpsc::Sender<()>>>>
}

impl ProgramBuilder {
  pub fn new(shader_root: PathBuf) -> Self {
    let (wsx, wrx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();
    let receivers: Arc<Mutex<BTreeMap<PathBuf, mpsc::Sender<()>>>> = Arc::new(Mutex::new(BTreeMap::new()));

    let _ = watcher.watch(shader_root);

    let receivers_ = receivers.clone();
    let _ = thread::spawn(move || {
      for event in wrx.iter() {
        match event {
          notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } => {
            if let Some(sx) = receivers_.lock().unwrap().get(&path) {
              sx.send(()).unwrap();
            }
          },
          _ => {}
        }
      }
    });

    ProgramBuilder {
      watcher: watcher,
      receivers: receivers
    }
  }

  pub fn retrieve<'a, T, GetUni>(&mut self, tess_path: Option<(PathBuf, PathBuf)>, vs_path: PathBuf, gs_path: Option<PathBuf>, fs_path: PathBuf, get_uni: GetUni) -> Result<WrappedProgram<'a, T>, ProgramError>
      where GetUni: 'a + Fn(ProgramProxy) -> Result<T, ProgramError> {
    let program = try!(new_program_from_disk(tess_path.clone(), vs_path.clone(), gs_path.clone(), fs_path.clone(), &get_uni));
    let (sx, rx) = mpsc::channel();

    self.monitor_shader(tess_path.clone(), vs_path.clone(), gs_path.clone(), fs_path.clone(), sx);

    let wrapped = WrappedProgram {
      rx: rx,
      program: program,
      get_uni: Box::new(get_uni),
      vs_path: vs_path,
      fs_path: fs_path,
      tess_path: tess_path,
      gs_path: gs_path,
    };

    Ok(wrapped)
  }

  /// Add surveillance of a given `Program` by providing the path to all its shaders. When a change
  /// occurs, the `Program` gets notified of the change via its `Receiver` channel part.
  pub fn monitor_shader(&mut self, tess: Option<(PathBuf, PathBuf)>, vs: PathBuf, gs: Option<PathBuf>, fs: PathBuf, sx: mpsc::Sender<()>) {
    let mut receivers = self.receivers.lock().unwrap();

    // vertex shader
    receivers.insert(vs, sx.clone());

    // fragment shader
    receivers.insert(fs, sx.clone());

    // tessellation, if needed
    if let Some((tcs, tes)) = tess {
      // tessellation control shader
      receivers.insert(tcs, sx.clone());
      // tessellation evaluation shader
      receivers.insert(tes, sx.clone());
    }

    // geometry shader, if needed
    if let Some(gs) = gs {
      receivers.insert(gs, sx.clone());
    }
  }
}

/// A `Program` wrapped by **ion**.
///
/// That wrapper is used to enable hot-reloading of shader programs.
pub struct WrappedProgram<'a, T> {
  rx: mpsc::Receiver<()>,
  program: Program<T>,
  get_uni: Box<Fn(ProgramProxy) -> Result<T, ProgramError> + 'a>,
  vs_path: PathBuf,
  fs_path: PathBuf,
  tess_path: Option<(PathBuf, PathBuf)>,
  gs_path: Option<PathBuf>
}

impl<'a, T> WrappedProgram<'a, T> {
  fn reload(&mut self) {
    let program = new_program_from_disk(self.tess_path.clone(), self.vs_path.clone(), self.gs_path.clone(), self.fs_path.clone(), &self.get_uni.as_ref());

    match program {
      Ok(program) => {
        self.program = program;
      },
      Err(err) => {
        warn!("reloading program has failed: {:?}", err);
      }
    }
  }

  /// Sync the embedded `Program`.
  pub fn sync(&mut self) {
    if self.rx.try_recv().is_ok() {
      self.reload();
    }
  }
}

impl<'a, T> Deref for WrappedProgram<'a, T> {
  type Target = Program<T>;

  fn deref(&self) -> &Self::Target {
    &self.program
  }
}
