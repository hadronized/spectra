use std::path::{Path, PathBuf};

/// Class of types that can be loaded.
pub trait Load: Sized {
  type Args;

  fn load<P>(path: P, args: Self::Args) -> Result<Self, LoadError> where P: AsRef<Path>;
}

/// Class of types that can be reloaded.
pub trait Reload: Load {
  fn reload<P>(&self, path :P) -> Result<Self, LoadError> where P: AsRef<Path>;
}

/// Default implementation for types which are loaded without any arguments.
impl<T> Reload for T where T: Load<Args=()> {
  fn reload<P>(&self, path :P) -> Result<Self, LoadError> where P: AsRef<Path> {
    Self::load(path, ())
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
  FileNotFound(PathBuf, String),
  ParseFailed(String),
  ConversionFailed(String)
}

#[cfg(feature = "hot-resource")]
mod hot {
  use notify::{self, RecommendedWatcher, Watcher};
  use std::collections::BTreeMap;
  use std::path::{Path, PathBuf};
  use std::sync::{Arc, Mutex};
  use std::sync::mpsc;
  use std::thread;

  /// Time to await after a resource update to establish that it should be reloaded.
  pub const UPDATE_AWAIT_TIME: f64 = 0.1; // 100ms

  pub struct ResourceManager {
    senders: Arc<Mutex<BTreeMap<PathBuf, mpsc::Sender<()>>>>
  }

  impl ResourceManager {
    pub fn new<P>(root: P) -> Self where P: AsRef<Path> {
      let (wsx, wrx) = mpsc::channel();
      let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();
      let senders: Arc<Mutex<BTreeMap<PathBuf, mpsc::Sender<()>>>> = Arc::new(Mutex::new(BTreeMap::new()));
      let senders_ = senders.clone();
      let root = root.as_ref().to_path_buf();

      let _ = thread::spawn(move || {
        let _ = watcher.watch(root);

        for event in wrx.iter() {
          match event {
            notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } => {
              if let Some(sx) = senders_.lock().unwrap().get(&path) {
                sx.send(()).unwrap();
              }
            },
            _ => {}
          }
        }
      });

      ResourceManager {
        senders: senders
      }
    }

    pub fn monitor<P>(&mut self, path: P, sx: mpsc::Sender<()>) where P: AsRef<Path> {
      let mut senders = self.senders.lock().unwrap();

      senders.insert(path.as_ref().to_path_buf(), sx);
    }
  }
}

#[cfg(not(feature = "hot-resource"))]
mod cold {
  use std::path::Path;
  use std::sync::mpsc;

  pub struct ResourceManager {}

  impl ResourceManager {
    pub fn new<P>(_: P) -> Self where P: AsRef<Path> {
      ResourceManager {}
    }

    pub fn monitor<P>(&mut self, _: P, _: mpsc::Sender<()>) where P: AsRef<Path> {}
  }
}

#[cfg(feature = "hot-resource")]
pub use self::hot::*;
#[cfg(not(feature = "hot-resource"))]
pub use self::cold::*;

/// Sync all the resources passed in as arguments.
#[macro_export]
macro_rules! sync {
  ($( $r:expr ),*) => {
    $( $r.sync(); )*
  }
}

/// A helper macro to declare a `sync` public method for a resource. The resource must
/// have a `last_update_time: Option<f64>` and a `reload(&mut self)` function.
#[cfg(feature = "hot-resource")]
#[macro_export]
macro_rules! decl_sync_hot {
  () => {
    pub fn sync(&mut self) {
      if self.rx.try_recv().is_ok() {
        self.last_update_time = Some(::time::precise_time_s());
      }

      match self.last_update_time {
        Some(last_update_time) if ::time::precise_time_s() - last_update_time >= ::resource::UPDATE_AWAIT_TIME => {
          self.reload();
          self.last_update_time = None;
        },
        _ => {}
      }
    }
  }
}
