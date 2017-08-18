// FIXME: add the support of transient objects

use any_cache::{Cache, HashCache};
pub use any_cache::CacheKey;
use notify::{Op, RawEvent, RecursiveMode, Watcher, raw_watcher};
use notify::op::WRITE;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

/// Loadable object from disk.
///
/// An object can be loaded from disk if given a path it can be output a `LoadResult<_>`. It’s
/// important to note that you’re not supposed to load objects directly from this trait. Instead,
/// you should use a `Store`.
pub trait Load: 'static + Sized {
  /// Key representing the resource. This type is used to uniquely identify a resource.
  type Key: CacheKey<Target = Self> + Clone + Debug;

  /// Convert from a key to its path representation.
  fn key_to_path(key: &Self::Key) -> PathBuf;

  /// Load a resource. The `Store` can be used to load or declare additional resource dependencies.
  /// The result type is used to register for dependency events.
  fn load<P>(path: P, cache: &mut Store) -> Result<LoadResult<Self>, LoadError> where P: AsRef<Path>;
}

/// Resource key. This type is used to adapt a key type’s target so that it can be mutably shared.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RKey<K>(K);

impl<K, T> CacheKey for RKey<K> where K: CacheKey<Target = T> {
  type Target = Rc<RefCell<T>>;
}

/// Result of a resource loading. This type enables you to register a resource for reloading events
/// of others (dependencies). If you don’t need to run specific code on a dependency reloading, use
/// the `.into()` function to lift your return value to `LoadResult<_>`.
pub struct LoadResult<T> {
  /// The loaded object.
  res: T,
  /// The list of dependencies to listen for events.
  dependencies: Vec<PathBuf>
}

impl<T> From<T> for LoadResult<T> {
  fn from(t: T) -> Self {
    LoadResult {
      res: t,
      dependencies: Vec::new()
    }
  }
}

/// Error that might occur while loading a resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
  /// The file was not found.
  FileNotFound(PathBuf),
  /// The file wasn’t correctly parsed.
  ParseFailed(String),
  /// The file wasn’t correctly converted, even though it might have been parsed.
  ConversionFailed(String)
}

/// Resources are wrapped in this type.
pub type Res<T> = Rc<RefCell<T>>;

/// Time to await after a resource update to establish that it should be reloaded.
const UPDATE_AWAIT_TIME_MS: u64 = 1000;

/// Resource store. Responsible for holding and presenting resources.
pub struct Store {
  // canonicalized root path of resources
  root: PathBuf,
  // resource cache
  cache: HashCache,
  // contains all metadata on resources
  metadata: HashMap<PathBuf, ResMetaData>,
  // dependencies, mapping a dependency to its observers
  dependencies: HashMap<PathBuf, PathBuf>,
  // vector of pairs (path, timestamp) giving indication on resources to reload
  dirty: Arc<Mutex<Vec<(PathBuf, Instant)>>>,
  #[allow(dead_code)]
  watcher_thread: thread::JoinHandle<()>
}

impl Store {
  /// Create a new store.
  pub fn new<P>(root: P) -> Result<Self, StoreError> where P: AsRef<Path> {
    let dirty: Arc<Mutex<Vec<(PathBuf, Instant)>>> = Arc::new(Mutex::new(Vec::new()));
    let dirty_ = dirty.clone();

    let root = root.as_ref().to_owned();
    let root_ = root.clone();
    let canon_root = root.canonicalize().map_err(|_| StoreError::RootDoesDotExit(root_.into()))?;
    let canon_root_ = canon_root.clone();
    let (wsx, wrx) = channel();
    let mut watcher = raw_watcher(wsx).unwrap();

    let join_handle = thread::spawn(move || {
      let _ = watcher.watch(canon_root_.clone(), RecursiveMode::Recursive);

      for event in wrx.iter() {
        match event {
          RawEvent { path: Some(ref path), op: Ok(op), .. } if op | WRITE != Op::empty() => {
            dirty_.lock().unwrap().push((path.strip_prefix(&canon_root_).unwrap().to_owned(), Instant::now()));
          },
          _ => ()
        }
      }
    });

    deb!("resource cache started and listens to file changes in {}", root.display());

    Ok(Store {
      root: canon_root,
      cache: HashCache::new(),
      metadata: HashMap::new(),
      dependencies: HashMap::new(),
      dirty: dirty,
      watcher_thread: join_handle
    })
  }

  /// Inject a new resource in the cache.
  ///
  /// `key` is used to cache the resource and `path` is the path to where to reload the
  /// resource.
  fn inject<T>(&mut self, key: &T::Key, resource: T, dependencies: Vec<PathBuf>) -> Res<T> where T: Load {
    // wrap the resource to make it shared mutably
    let res = Rc::new(RefCell::new(resource));
    let res_ = res.clone();

    // create the path associated with the given key
    let key_ = key.clone();
    let path = self.root.join(T::key_to_path(&key));
    let path_ = path.clone();

    // closure used to reload the object when needed
    let on_reload: Box<for<'a> Fn(&'a mut Store) -> Result<(), LoadError>> = Box::new(move |cache| {
      deb!("reloading {:?}", key_);

      match T::load(&path_, cache) {
        Ok(load_result) => {
          // replace the current resource with the freshly loaded one
          *res_.borrow_mut() = load_result.res;
          deb!("reloaded {:?}", key_);
          Ok(())
        },
        Err(e) => {
          warn!("{:?} failed to reload:\n{:#?}", key_, e);
          Err(e)
        }
      }
    });

    let metadata = ResMetaData {
      on_reload: on_reload,
      last_update_instant: Instant::now(),
    };


    // cache the resource and its meta data
    self.cache.save(RKey(key.clone()), res.clone());
    self.metadata.insert(path.clone(), metadata);

    deb!("cached resource {:?}", key);

    // register the resource as an observer of its dependencies in the dependencies graph
    for dep_key in dependencies {
      self.dependencies.insert(dep_key, path.clone());
    }

    res
  }

  /// Get a resource from the cache and return an error if loading failed.
  fn get_<T>(&mut self, key: &T::Key) -> Result<Res<T>, LoadError> where T: Load {
    let rekey: RKey<T::Key> = RKey(key.clone());
    match self.cache.get(&rekey).cloned() {
      Some(resource) => {
        deb!("cache hit for {:?}", key);
        Ok(resource)
      },
      None => {
        deb!("cache miss for {:?}", key);

        // specific loading
        info!("loading {:?}", key);
        let path = self.root.join(T::key_to_path(key));
        let load_result = T::load(&path, self)?;
        Ok(self.inject(key, load_result.res, load_result.dependencies))
      }
    }
  }

  /// Get a resource from the cache for the given key.
  pub fn get<T>(&mut self, key: &T::Key) -> Option<Res<T>> where T: Load {
    deb!("getting {:?}", key);

    match self.get_(key) {
      Ok(resource) => Some(resource),
      Err(e) => {
        err!("cannot get {:?} because:\n{:#?}", key, e);
        None
      }
    }
  }

  /// Get a resource from the store for the given key. If it fails, a proxed version is used, which
  /// will get replaced by the resource once it’s available.
  pub fn get_proxied<T, P>(&mut self, key: &T::Key, proxy: P) -> Result<Res<T>, LoadError>
      where T: Load,
            P: FnOnce() -> T {
    match self.get_(key) {
      Ok(resource) => Ok(resource),
      Err(e) => {
        warn!("proxied {:?} because:\n{:#?}", key, e);

        // FIXME: we set the dependencies to none here, which is silly; find a better design
        Ok(self.inject(key, proxy(), Vec::new()))
      }
    }
  }

  /// Synchronize the cache by updating the resources that ought to.
  pub fn sync(&mut self) {
    let dirty = self.dirty.clone();
    let mut dirty_ = dirty.lock().unwrap();

    for &(ref path, ref instant) in dirty_.iter() {
      if let Some(mut metadata) = self.metadata.remove(path) {
        if instant.duration_since(metadata.last_update_instant) >= Duration::from_millis(UPDATE_AWAIT_TIME_MS) {
          if (metadata.on_reload)(self).is_ok() {
            // if we have successfully reloaded the resource, notify the observers that this
            // dependency has changed
            for dep in self.dependencies.get(path.as_path()).cloned() {
              if let Some(obs_metadata) = self.metadata.remove(dep.as_path()) {
                if let Err(e) = (obs_metadata.on_reload)(self) {
                  warn!("cannot reload {:?} {:?}", dep, e);
                }

                self.metadata.insert(dep, obs_metadata);
              }
            }
          }
        }

        metadata.last_update_instant = *instant;
        self.metadata.insert(path.clone(), metadata);
      }
    }

    dirty_.clear();
  }
}

/// Meta data about a resource.
struct ResMetaData {
  on_reload: Box<Fn(&mut Store) -> Result<(), LoadError>>,
  last_update_instant: Instant,
}

/// Error that might happen when creating a resource cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreError {
  /// The root path for the resources was not found.
  RootDoesDotExit(PathBuf)
}
