//! Shader module.
//!
//! Shader functions and declarations can be grouped in so-called *modules*. Modules structure is
//! inherently tied to the filesystem’s tree.
//!
//! You’re not supposed to use modules at the Rust level, even though you can. You’re supposed to
//! actually write modules that will be used by shader programs.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use render::shader::lang::parser;
use render::shader::lang::syntax::{Module as SyntaxModule, ModulePath};
use sys::resource::{CacheKey, Load, LoadError, LoadResult, Store, StoreKey};

// FIXME: see where to put that shit; especially, ModuleKey should be implemented with ModuleKe(ModulePath), LOL
fn module_path_to_module_key(mp: &ModulePath) -> ModuleKey {
  ModuleKey(mp.path.join("/") + ".spsl")
}

/// Shader module.
///
/// A shader module is a piece of GLSL code with optional import lists (dependencies).
///
/// You’re not supposed to directly manipulate any object of this type. You just write modules on
/// disk and let everyting happen automatically for you.
#[derive(Clone, Debug, PartialEq)]
pub struct Module(SyntaxModule);

impl Module {
  /// Retrieve all the modules this module depends on, without duplicates.
  pub fn deps(&self, store: &mut Store, path: &ModulePath) -> Result<Vec<ModulePath>, DepsError> {
    let mut deps = Vec::new();
    self.deps_no_cycle(store, &path, &mut Vec::new(), &mut deps).map(|_| deps)
  }

  fn deps_no_cycle(&self, store: &mut Store, path: &ModulePath, parents: &mut Vec<ModulePath>, deps: &mut Vec<ModulePath>) -> Result<(), DepsError> {
    let imports = self.0.imports.iter().map(|il| &il.module);

    parents.push(path.clone());

    for module_name in imports {
      // check whether it’s already in the deps
      if deps.contains(module_name) {
        continue;
      }

      // check whether the module was already visited
      if parents.contains(module_name) {
        return Err(DepsError::Cycle(path.clone(), module_name.clone()));
      }

      // get the dependency module 
      let module = store.get(&module_path_to_module_key(&module_name)).ok_or_else(|| DepsError::LoadError(module_name.clone()))?;
      let r = module.borrow().deps_no_cycle(store, module_name, parents, deps)?;

      deps.push(module_name.clone());
      parents.pop();
    }

    Ok(())
  }
}

/// Class of errors that can happen in dependencies.
#[derive(Clone, Debug, PartialEq)]
pub enum DepsError {
  /// If a module’s dependencies has any cycle, the dependencies are unusable and the cycle is
  /// returned.
  Cycle(ModulePath, ModulePath),
  /// There was a loading error of a module.
  LoadError(ModulePath)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleKey(String);

impl ModuleKey {
  pub fn new(key: &str) -> Self {
    ModuleKey(key.to_owned())
  }
}

impl CacheKey for ModuleKey {
  type Target = Module;
}

impl StoreKey for ModuleKey {
  fn key_to_path(&self) -> PathBuf {
    self.0.clone().into()
  }
}

impl Load for Module {
  fn load<P>(path: P, _: &mut Store) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path> {
    let path = path.as_ref();

    let mut fh = File::open(path).map_err(|_| LoadError::FileNotFound(path.to_owned()))?;
    let mut src = String::new();
    let _ = fh.read_to_string(&mut src);

    match parser::parse_str(&src[..], parser::module) {
      parser::ParseResult::Ok(module) => {
        Ok(Module(module).into())
      }
      parser::ParseResult::Err(e) => Err(LoadError::ConversionFailed(format!("{:?}", e))),
      _ => Err(LoadError::ConversionFailed("incomplete input".to_owned()))
    }
  }
}

