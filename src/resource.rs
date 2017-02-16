// FIXME: add the support of transient objects

use any_cache::{self, Cache, HashCache};
use notify::{self, RecommendedWatcher, Watcher};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use time::precise_time_s;

use model::Model;
use object::Object;
use shader::Program;
use spline::Spline;
use texture::TextureImage;

/// Class of types that can be loaded.
pub trait Load: Sized {
  /// Arguments passed at loading.
  type Args: Clone;

  /// Static string representing the name of the type (used in filesystem caching).
  const TY_STR: &'static str;

  fn load<P>(path: P, cache: &mut ResCache, args: Self::Args) -> Result<Self> where P: AsRef<Path>;
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
#[derive(Debug)]
pub struct Res<T>(Rc<RefCell<T>>);

impl<T> Clone for Res<T> {
  fn clone(&self) -> Self {
    Res(self.0.clone())
  }
}

impl<T> Deref for Res<T> {
  type Target = RefCell<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

type Timestamp = f64;

/// Time to await after a resource update to establish that it should be reloaded.
const UPDATE_AWAIT_TIME: Timestamp = 1.; // 1s

/// Resource cache. Responsible for caching resource.
pub struct ResCache {
  // contains all the typed-erased Rc<RefCell<T>>
  cache: HashCache<PathBuf>,
  // contains all metadata on resources
  metadata: HashMap<PathBuf, ResMetaData>,
  // vector of pair (path, timestamp) giving indication on resources to reload
  dirty: Arc<Mutex<Vec<(PathBuf, Timestamp)>>>,
  watcher_thread: thread::JoinHandle<()>
}

struct ResMetaData {
  on_reload: Box<Fn(&mut ResCache)>,
  last_update_timestamp: Timestamp // timestamp of the last update
}

impl ResCache {
  /// Create a new cache.
  pub fn new<P>(root: P) -> Self where P: AsRef<Path> {
    let dirty: Arc<Mutex<Vec<(PathBuf, Timestamp)>>> = Arc::new(Mutex::new(Vec::new()));
    let dirty_ = dirty.clone();

    let root = root.as_ref().to_owned();
    let (wsx, wrx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();

    let join_handle = thread::spawn(move || {
      let _ = watcher.watch(root);

      for event in wrx.iter() {
        if let notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } = event {
          dirty_.lock().unwrap().push((path.clone(), precise_time_s()));
        }
      }
    });

    ResCache {
      cache: HashCache::new(),
      metadata: HashMap::new(),
      dirty: dirty,
      watcher_thread: join_handle
    }
  }

  /// Get a resource from the cache.
  pub fn get<T>(&mut self, key: &str, args: T::Args) -> Option<Res<T>> where T: 'static + Any + Reload {
    let path_str = format!("data/{}/{}", T::TY_STR, key);
    let path = Path::new(&path_str);
    let path_buf = path.to_owned();

    match self.cache.get::<Res<T>>(&path_buf).cloned() {
      r@Some(..) => {
        deb!("cache hit for {} ({})", key, path_str);
        r
      },
      None => {
        deb!("cache miss for {} ({})", key, path_str);

        // specific loading
        if path.exists() {
          match T::load(&path, self, args.clone()) {
            Ok(resource) => {
              let res = Res(Rc::new(RefCell::new(resource)));
              let res_ = res.clone();

              let path_buf_ = path_buf.clone();
              // closure used to reload the object when needed
              let on_reload: Box<for<'a> Fn(&'a mut ResCache)> = Box::new(move |cache_| {
                match T::load(&path_buf_, cache_, args.clone()) {
                  Ok(new_resource) => {
                    // replace the current resource with the freshly loaded one
                    *res_.borrow_mut() = new_resource;
                    deb!("reloaded resource from {:?}", path_buf_);
                  },
                  Err(e) => {
                    warn!("reloading resource from {:?} has failed:\n{:#?}", path_buf_, e);
                  }
                }
              });

              let metadata = ResMetaData {
                on_reload: on_reload,
                last_update_timestamp: precise_time_s()
              };

              // cache the resource and its meta data
              self.cache.save(path_buf.clone(), res.clone());
              self.metadata.insert(path_buf, metadata);

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

  // TODO: maybe we should have a ResCache::update() instead of get() + save()?
  ///// Synchronize the cache by updating the resource that ought to.
  //pub fn sync(&mut self) {
  //  let mut dirty = self.dirty.lock().unwrap();

  //  for &(ref path, ref timestamp) in dirty.iter() {
  //    let mut cache_entry = self.cache.get::<ResCacheEntry>(&path).cloned().unwrap();

  //    if timestamp - cache_entry.last_update_timestamp >= UPDATE_AWAIT_TIME {
  //      cache_entry.on_reload();
  //    }

  //    cache_entry.last_update_timestamp = timestamp;
  //    self.cache.save(path.clone(), cache_entry);
  //  }

  //  dirty.clear();
  //}
}
