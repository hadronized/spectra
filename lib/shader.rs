use luminance::{FragmentShader, GeometryShader, StageError, ShaderTypeable,
                TessellationControlShader, TessellationEvaluationShader, VertexShader};
use luminance_gl::gl33::{ProgramProxy, Stage};
use std::default::Default;
use std::fs;
use std::io::Read;
use std::path::Path;
use resource::*;

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

fn read_stage<T>(path: &str) -> Result<Stage<T>, StageError> where T: ShaderTypeable {
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

pub fn get_program<T, GetUni: Fn(ProgramProxy) -> Result<T, ProgramError>>(manager: &mut ProgramManager, name: &str, force: bool, get_uni: GetUni) -> Result<Program<T>, ShaderError> {
  let root = format!("rt/shaders/{}", name);
  let vs_src = &format!("{}/vertex.glsl", root);
  let fs_src = &format!("{}/fragment.glsl", root);
  let tcs_src = &format!("{}/tess_ctrl.glsl", root);
  let tes_src = &format!("{}/tess_eval.glsl", root);
  let gs_src = &format!("{}/geometry.glsl", root);

  info!("loading shader program: \x1b[35m{} ", name);

  // we need at least a vertex and a fragment shader
  if !Path::new(vs_src).is_file() {
    return stage_error(StageError::CompilationFailed(VertexShader::shader_type(), String::from(format!("cannot find {}", vs_src))));
  }

  if !Path::new(fs_src).is_file() {
    return stage_error(StageError::CompilationFailed(FragmentShader::shader_type(), String::from(format!("cannot find {}", fs_src))));
  }

  let vs = Stage::<VertexShader>::load(&mut manager.vs_manager, vs_src, force);
  if let Err(err) = vs { return Err(err) };

  let fs = Stage::<FragmentShader>::load(&mut manager.fs_manager, fs_src, force);
  if let Err(err) = fs { return Err(err) };

  // check whether we have tessellation shaders
  let has_tcs = Path::new(tcs_src).is_file();
  let has_tes = Path::new(tes_src).is_file();

  let tess = if has_tcs && !has_tes {
    return stage_error(StageError::CompilationFailed(TessellationEvaluationShader::shader_type(), String::from(format!("cannot find {}", tes_src))));
  } else if !has_tcs && has_tes {
    return stage_error(StageError::CompilationFailed(TessellationControlShader::shader_type(), String::from(format!("cannot find {}", tcs_src))));
  } else if has_tcs && has_tes {
    let tcs = Stage::<TessellationControlShader>::load(&mut manager.tcs_manager, tcs_src, force);
    if let Err(err) = tcs { return Err(err) };

    let tes = Stage::<TessellationEvaluationShader>::load(&mut manager.tes_manager, tes_src, force);
    if let Err(err) = tes { return Err(err) };

    Some((tcs.unwrap(), tes.unwrap()))
  } else {
    None
  };

  // check whether we have a geometry shader
  let gs = if Path::new(gs_src).is_file() {
    let gs = Stage::<GeometryShader>::load(&mut manager.gs_manager, gs_src, force);
    if let Err(err) = gs { return Err(err) };

    Some(gs.unwrap())
  } else {
    None
  };

  match (tess, gs) {
    (None, Some(gs)) => {
      match Program::new(None, &*vs.unwrap().upgrade().unwrap(), Some(&*gs.upgrade().unwrap()), &*fs.unwrap().upgrade().unwrap(), get_uni) {
        Ok(program) => Ok(program),
        Err(err) => program_error(err)
      }
    },
    (Some((tcs, tes)), None) => {
      match Program::new(Some((&*tcs.upgrade().unwrap(), &*tes.upgrade().unwrap())), &*vs.unwrap().upgrade().unwrap(), None, &*fs.unwrap().upgrade().unwrap(), get_uni) {
        Ok(program) => Ok(program),
        Err(err) => program_error(err)
      }
    },
    (Some((tcs, tes)), Some(gs)) => {
      match Program::new(Some((&*tcs.upgrade().unwrap(), &*tes.upgrade().unwrap())), &*vs.unwrap().upgrade().unwrap(), Some(&*gs.upgrade().unwrap()), &*fs.unwrap().upgrade().unwrap(), get_uni) {
        Ok(program) => Ok(program),
        Err(err) => program_error(err)
      }
    },
    _ => {
      match Program::new(None, &*vs.unwrap().upgrade().unwrap(), None, &*fs.unwrap().upgrade().unwrap(), get_uni) {
        Ok(program) => Ok(program),
        Err(err) => program_error(err)
      }
    }
  }
}

#[derive(Debug)]
pub enum ShaderError {
  StageError(StageError),
  ProgramError(ProgramError)
}

fn stage_error<T>(e: StageError) -> Result<T, ShaderError> {
  Err(ShaderError::StageError(e))
}

fn program_error<T>(e: ProgramError) -> Result<T, ShaderError> {
  Err(ShaderError::ProgramError(e))
}

impl<T> Resource for Stage<T> where T: ShaderTypeable {
  type Manager = ManagerMap<Self>;
  type Error = ShaderError;

  fn load(manager: &mut Self::Manager, name: &str, force: bool) -> Result<Managed<Self>, Self::Error> {
    cache_fetch!(manager, name, force);

    read_stage(name).map(|stage| {
      let arc = sync::Arc::new(stage);
      manager.insert(String::from(name), arc.clone());
      sync::Arc::downgrade(&arc)
    }).map_err(|e| ShaderError::StageError(e) )
  }

  fn unload(manager: &mut Self::Manager, name: &str) {
    let _ = manager.remove(name);
  }

  fn reload(manager: &mut Self::Manager, name: &str) -> Result<(), Self::Error> {
    Self::load(manager, name, true).map(|_| ())
  }
}

pub struct ProgramManager {
  //program_manager: ManagerMap<Program>,
  tcs_manager: <Stage<TessellationControlShader> as Resource>::Manager,
  tes_manager: <Stage<TessellationEvaluationShader> as Resource>::Manager,
  vs_manager: <Stage<VertexShader> as Resource>::Manager,
  gs_manager: <Stage<GeometryShader> as Resource>::Manager,
  fs_manager: <Stage<FragmentShader> as Resource>::Manager
}

impl Default for ProgramManager {
  fn default() -> Self {
    ProgramManager {
      //program_manager: Default::default(),
      tcs_manager: Default::default(),
      tes_manager: Default::default(),
      vs_manager: Default::default(),
      gs_manager: Default::default(),
      fs_manager: Default::default()
    }
  }
}

//impl Resource for Program {
//  type Manager = ProgramManager;
//  type Error = ShaderError;
//
//  fn load(manager: &mut Self::Manager, name: &str, force: bool) -> Result<Managed<Self>, Self::Error> {
//    cache_fetch!(manager.program_manager, name, force);
//
//    get_program(manager, name, force).map(|program| {
//      let arc = sync::Arc::new(program);
//      manager.program_manager.insert(String::from(name), arc.clone());
//      sync::Arc::downgrade(&arc)
//    })
//  }
//
//  fn unload(manager: &mut Self::Manager, name: &str) {
//    default_unload_impl!(manager.program_manager, name);
//  }
//
//  fn reload(manager: &mut Self::Manager, name: &str) -> Result<(), Self::Error> {
//    get_program(manager, name, true).map(|program| {
//      let arc = manager.program_manager.get_mut(name).unwrap();
//      sync::Arc::get_mut(arc).map(|p| *p = program);
//    })
//  }
//}
