// FIXME: add the support of transient objects

use any_cache::{self, HashCache};
use notify::{self, RecommendedWatcher, Watcher};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use time::precise_time_s;

use id::Id;
use model::Model;
use object::Object;
use shader::Program;
use spline::Spline;
use texture::TextureImage;

/// Class of types that can be loaded.
pub trait Load: Sized {
  /// Arguments passed at loading.
  type Args;

  /// Static string representing the name of the type (used in filesystem caching).
  const TY_STR: &'static str;

  fn load<P>(path: P, cache: &mut Cache, args: Self::Args) -> Result<Self> where P: AsRef<Path>;
}

/// Class of types that can be reloaded.
///
/// The idea is to simply recover the arguments used in `Load::load`.
pub trait Reload: Load {
  fn reload_args(&self) -> Self::Args;
}

/// Default implementation for types which are loaded without any arguments.
impl<T> Reload for T where T: Load<Args=()> {
  fn reload_args(&self) -> Self::Args {
    ()
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
  FileNotFound(PathBuf, String),
  ParseFailed(String),
  ConversionFailed(String)
}

pub type Result<T> = ::std::result::Result<T, LoadError>;

/// Resources are wrapped in this type.
pub struct Res<T>(Rc<RefCell<T>>);

impl<T> Deref for Res<T> {
  type Target = Rc<RefCell<T>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

type Timestamp = f64;

/// Time to await after a resource update to establish that it should be reloaded.
const UPDATE_AWAIT_TIME: Timestamp = 1.; // 1s

/// Resource cache. Responsible for caching resource.
pub struct Cache {
  cache: HashCache,
  // vector of pair (path, timestamp) giving indication on resources to reload
  dirty: Arc<Mutex<Vec<(PathBuf, Timestamp)>>,
  watcher_thread: thread::JoinHandle<()>
}

/// Entry in the resource cache corresponding to a given resource which type is `T`.
struct CacheEntry {
  on_reload: Box<for<'a> Fn(&'a mut Cache)>,
  last_update_timestamp: Timestamp // timestamp of the last update
}

impl Cache {
  /// Create a new cache.
  pub fn new<P>(root: P) -> Self where P: AsRef<Path> {
    let dirty: Arc<Mutex<Vec<(PathBuf, Timestamp>>> = Arc::new(Mutex::new(Vec::new()));
    let dirty_ = dirty.clone();

    let root = root.as_ref().to_owned();
    let (wsx, wrx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();

    let join_handle = thread::spawn(move || {
      let _ = watcher.watch(root);

      for event in wrx.iter() {
        if let notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } = event {
          dirty.lock().unwrap().push((path.clone(), precise_time_s()));
        }
      }
    });

    Cache {
      cache: HashCache::new(),
      dirty: dirty
      watcher_thread: join_handle
    }
  }

  /// Get a resource from the cache.
  pub fn get<T>(&mut self, key: &str, args: T::Args) -> Option<Res<T>> where T: 'static + Any + Reload {
    let path_str = format!("data/{}/{}", T::TY_STR, key);
    let path = Path::new(&path_str);

    match self.cache.get::<CacheEntry<T>>(key) {
      Some(entry) => {
        deb!("cache hit for {} ({})", key, path_str);
        entry.resource.clone()
      },
      None => {
        deb!("cache miss for {} ({})", key, path_str);

        // specific loading
        if path.exists() {
          match T::load(&path, self, args) {
            Ok(resource) => {
              let path_buf = path.to_owned();

              let res = Res(Rc::new(RefCell::new(resource)));
              let res_ = res.clone();

              let path_buf_ = path_buf.clone();
              // closure used to reload the object when needed
              let on_reload = Box::new(move |cache| {
                match T::load(&path_buf_, cache, args) {
                  Ok(new_resource) => {
                    // replace the current resource with the freshly loaded one
                    *res_.borrow_mut() = resource;
                    deb!("reloaded resource from {:?}", path_buf);
                  },
                  Err(e) => {
                    warn!("reloading resource from {:?} has failed:\n{:#?}", path, e);
                  }
                }
              });

              // cache entry
              let entry = CacheEntry {
                on_reload: on_reload,
                last_update_timestamp: precise_time_s()
              };

              // cache the resource and return it
              self.cache.save(path_buf, entry);
              Some(res)
            },
            Err(e) => {
              err!("unable to load resource from {}:\n{:#?}", path_str, e);
              None
            }
          }
        } else { // path doesn’t exist
          err!("resource at {} cannot be found", path_str);
          None
        }
      }
    }
  }

  // TODO: maybe we should have a Cache::update() instead of get() + save()?
  /// Synchronize the cache by updating the resource that ought to.
  pub fn sync(&mut self) {
    for (path, timestamp) in self.dirty.lock().unwrap().iter() {
      let mut cache_entry = self.cache.get(&path).cloned().unwrap();

      if timestamp - cache_entry.last_update_timestamp >= UPDATE_AWAIT_TIME {
        cache_entry.on_reload();
      }

      new_cache_entry.last_update_timestamp = timestamp;
      self.cache.save(&path, new_cache_entry);
    }

    self.dirty.clear();
  }
}
