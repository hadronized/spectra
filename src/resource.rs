// FIXME: add the support of transient objects

use any_cache::{Cache, HashCache};
use notify::{self, RecommendedWatcher, Watcher};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

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

impl<T> Res<T> {
  pub fn new(resource: T) -> Self {
    Res(Rc::new(RefCell::new(resource)))
  }

  pub fn as_rc(&self) -> &Rc<RefCell<T>> {
    &self.0
  }
}

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

impl<T> From<Rc<RefCell<T>>> for Res<T> {
  fn from(rc: Rc<RefCell<T>>) -> Self {
    Res(rc)
  }
}

/// Time to await after a resource update to establish that it should be reloaded.
const UPDATE_AWAIT_TIME_MS: u64 = 1000;

/// Resource cache. Responsible for caching resource.
pub struct ResCache {
  // contains all the typed-erased Rc<RefCell<T>>
  cache: HashCache<PathBuf>,
  // contains all metadata on resources
  metadata: HashMap<PathBuf, ResMetaData>,
  // vector of pair (path, timestamp) giving indication on resources to reload
  dirty: Arc<Mutex<Vec<(PathBuf, Instant)>>>,
  #[allow(dead_code)]
  watcher_thread: thread::JoinHandle<()>
}

struct ResMetaData {
  on_reload: Box<Fn(&mut ResCache)>,
  last_update_instant: Instant
}

impl ResCache {
  /// Create a new cache.
  pub fn new<P>(root: P) -> Self where P: AsRef<Path> {
    let dirty: Arc<Mutex<Vec<(PathBuf, Instant)>>> = Arc::new(Mutex::new(Vec::new()));
    let dirty_ = dirty.clone();

    let root = root.as_ref().to_owned();
    let (wsx, wrx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();

    let join_handle = thread::spawn(move || {
      let _ = watcher.watch(root);

      for event in wrx.iter() {
        if let notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } = event {
          dirty_.lock().unwrap().push((path.clone(), Instant::now()));
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

  /// Inject a new resource in the cache.
  fn inject<T>(&mut self, path_buf: &PathBuf, resource: T, args: T::Args) -> Res<T> where T: 'static + Any + Reload {
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
      last_update_instant: Instant::now()
    };

    // cache the resource and its meta data
    self.cache.save(path_buf.clone(), res.clone());
    self.metadata.insert(path_buf.clone(), metadata);

    res
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
              Some(self.inject(&path_buf, resource, args))
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

//   /// Get a resource from the cache. If it fails, a proxy version of it is used, which will get
//   /// replaced by the resource once it’s avaible.
//   pub fn get_proxied<T, P>(&mut self, key: &str, args: T::Args, proxy: P) -> Option<Res<T>>
//       where T: 'static + Any + Reload,
//             P: FnOnce() -> T {
//     // riterite
//     self.get::<T>().unwrap_or_else(|| {
//       deb!("proxying resource {}", key);
// 
// 
//     })
//   }

  /// Synchronize the cache by updating the resource that ought to.
  pub fn sync(&mut self) {
    let dirty = self.dirty.clone();
    let mut dirty_ = dirty.lock().unwrap();

    for &(ref path, ref instant) in dirty_.iter() {
      let mut metadata = self.metadata.remove(path).unwrap();

      if instant.duration_since(metadata.last_update_instant) >= Duration::from_millis(UPDATE_AWAIT_TIME_MS) {
        (metadata.on_reload)(self);
      }

      metadata.last_update_instant = *instant;
      self.metadata.insert(path.clone(), metadata);
    }

    dirty_.clear();
  }
}
