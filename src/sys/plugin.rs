use cargo::core::package::Package;
use cargo::util::config::Config;
use libloading::Library;
use std::error::Error;
use std::fmt;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8_unchecked;
use tempdir::TempDir;

use render::pipeline::Gpu;
use sys::resource::{DebugRes, Load, Loaded, PathKey, Storage, load_with};

pub struct Plugin {
  #[allow(dead_code)]
  lib: Library,
  demo: Box<Demo>
}

impl DebugRes for Plugin {
  const TY_DESC: &'static str = "plugin";
}

impl Load for Plugin {
  type Key = PathKey;

  type Error = PluginError;

  fn load(key: Self::Key, storage: &mut Storage) -> Result<Loaded<Self>, Self::Error> {
    let path = key.as_path();

    load_with::<Self, _, _>(path, move || {
      let tmp_dir = TempDir::new("").map_err(PluginError::CannotCreateTempDir)?;
      let lib_path = tmp_dir.path().join("a.splugin");

      compile_rs(path, &lib_path)?;

      let lib = Library::new(&lib_path).map_err(PluginError::CannotOpenPlugin)?;

      // get the demo getter in the library
      let demo = unsafe {
        let sym = lib.get::<GetDemo>(b"spectra_plugin");
        // get the boxed demo
        sym.map(|getter| getter(storage)).map_err(PluginError::CannotFindSymbol)?
      };

      let plugin = Plugin { lib, demo };

      Ok(plugin.into())
    })
  }

  impl_reload_passthrough!();
}

impl Deref for Plugin {
  type Target = Box<Demo>;

  fn deref(&self) -> &Self::Target {
    &self.demo
  }
}

impl DerefMut for Plugin {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.demo
  }
}

#[derive(Debug)]
pub enum PluginError {
  CannotCreateTempDir(io::Error),
  CannotOpenPlugin(io::Error),
  CannotFindSymbol(io::Error),
  CompilationFailed(String),
  CannotFindMetaData(String)
}

impl fmt::Display for PluginError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for PluginError {
  fn description(&self) -> &str {
    match *self {
      PluginError::CannotCreateTempDir(_) => "cannot create temporary directory",
      PluginError::CannotOpenPlugin(_) => "cannot open plugin",
      PluginError::CannotFindSymbol(_) => "cannot find symbol",
      PluginError::CompilationFailed(_) => "compilation failed",
      PluginError::CannotFindMetaData(_) => "cannot find metadata"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      PluginError::CannotCreateTempDir(ref e) => Some(e),
      PluginError::CannotOpenPlugin(ref e) => Some(e),
      PluginError::CannotFindSymbol(ref e) => Some(e),
      _ => None
    }
  }
}

pub trait Demo {
  fn render(&mut self, gpu: &Gpu, t: f32);
}

pub type GetDemo = extern fn(&mut Storage) -> Box<Demo>;

/// Compile a Rust module and output, if compilation has succeeded, a shared library in a temporary
/// directory.
///
/// This function requires rustc to be in scope.
fn compile_rs<P>(rs_path: P, target_path: P) -> Result<(), PluginError> where P: AsRef<Path> {
  let rs_path = rs_path.as_ref();
  let rs_os_path = rs_path.as_os_str();
  let target_path = target_path.as_ref();
  let target_os_path = target_path.as_os_str();

  let root_path = find_project_root()?;
  let build_ty = if cfg!(debug_assertions) { "debug" } else { "release" };

  let deps_path = root_path.join("target").join(build_ty).join("deps");
  let deps_arg = format!("dependency={}", deps_path.display());

  let pkg = package_for(&root_path)?;

  let mut cmd = Command::new("rustc");
  cmd.arg("--crate-type")
     .arg("dylib")
     .arg(rs_os_path)
     .arg("-o")
     .arg(target_os_path)
     .arg("-L")
     .arg(&deps_arg);

  add_rustc_cmd_crates(&pkg, &deps_path, &mut cmd)?;

  let result = cmd.output().unwrap();

  if !result.status.success() {
    let stderr = unsafe { from_utf8_unchecked(result.stderr.as_slice()) };
    Err(PluginError::CompilationFailed(stderr.to_owned()))
  } else {
    Ok(())
  }
}

/// Find the project root path so that we can pick dependencies.
fn find_project_root() -> Result<PathBuf, PluginError> {
  let result =
    Command::new("cargo")
    .arg("locate-project")
    .output()
    .unwrap();

  if !result.status.success() {
    Err(PluginError::CannotFindMetaData("cannot locate root project".to_owned()))
  } else {
    let json = unsafe { from_utf8_unchecked(result.stdout.as_slice()) };
    let root =
      json.split(':').nth(1).and_then(|x| {
        if x.len() >= 3 {
          Path::new(&x[1..x.len()-3]).parent().map(|x| x.to_owned())
        } else {
          None
        }
      });

    root.ok_or_else(|| PluginError::CannotFindMetaData("cannot extract root project path from metadata".to_owned()))
  }
}

/// Generate a Package for a given root path.
fn package_for(path: &Path) -> Result<Package, PluginError> {
  let toml_path = path.join("Cargo.toml");
  let cfg = Config::default().map_err(|_| PluginError::CannotFindMetaData("cannot create cargo config".to_owned()))?;

  Package::for_path(&toml_path, &cfg).map_err(|_| PluginError::CannotFindMetaData("cannot generate package".to_owned()))
}

/// Generate the list of extern= arguments to pass to rustc.
fn extern_rustc_args(pkg: &Package, deps_root: &Path) -> Result<Vec<String>, PluginError> {
  // first, find local crates
  let locals = pkg.targets().into_iter().filter(|tgt| tgt.is_lib()).filter_map(|target| {
    //let crate_path = find_crate(target.name(), &deps_root)?;
    find_crate(target.name(), &deps_root).ok().map(|crate_path| {
      format!("{}={}", target.name(), crate_path.display())
    })
  });

  // then, include “real” external ones
  let externals = pkg.dependencies().into_iter().filter_map(|dep| {
    find_crate(dep.name(), &deps_root).ok().map(|crate_path| {
      format!("{}={}", dep.name(), crate_path.display())
    })
  });

  Ok(locals.chain(externals).collect())
}

/// Add the crates list to a Command (representing a rustc invokation).
fn add_rustc_cmd_crates(pkg: &Package, deps_root: &Path, cmd: &mut Command) -> Result<(), PluginError> {
  let args = extern_rustc_args(pkg, deps_root)?;

  for arg in args {
    cmd.arg("--extern");
    cmd.arg(&arg);
  }

  Ok(())
}

/// Find the given crate in the given directory.
fn find_crate(crate_name: &str, path: &Path) -> Result<PathBuf, PluginError> {
  if let Ok(dir) = path.read_dir() {
    for entry in dir {
      let path = entry.unwrap().path();

      match path.file_name() {
        Some(filename) => {
          let lib_name = format!("lib{}-", crate_name);
          let filename = filename.to_str().unwrap();

          if filename.starts_with(&lib_name) && filename.ends_with(".rlib") {
            return Ok(path.to_owned());
          }
        }

        _ => ()
      }
    }

    Err(PluginError::CannotFindMetaData(format!("cannot find the {} crate", crate_name)))
  } else {
    Err(PluginError::CannotFindMetaData("cannot find read dependencies".to_owned()))
  }
}
