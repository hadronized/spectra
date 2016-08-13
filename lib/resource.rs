use shader::ProgramManager;
use std::collections::BTreeMap;
use std::default::Default;

pub use std::sync;

/// A `Managed<T>` represents a resource which type is `T`. That resource is handled by an external
/// handler.
pub type Managed<T> = sync::Weak<T>;

pub type ManagerMap<T> = BTreeMap<String, sync::Arc<T>>;

/// A resource is a value that is handled by external stimuli – commonly, filesystem changes. It
/// must have a *unique ID*, used to identify that object in a unique way.
pub trait Resource: Sized {
  /// A type that can handle resource.
  type Manager;
  /// Type of error that can be raised during resource acquisition.
  type Error;

  /// Load a resource via its name. The `force` parameter forces the resource to be cleaned out of
  /// any caching system and reloaded afterwards.
  ///
  /// # Failures
  ///
  /// On a load failure, this function returns an error with a value of type `ResourceError`.
  fn load(manager: &mut Self::Manager, name: &str, force: bool) -> Result<Managed<Self>, Self::Error>;
  /// Unload a resource. That function will force the unloading of the resource if it’s not released
  /// yet. It’s also called when all `Managed` objects are dropped.
  fn unload(manager: &mut Self::Manager, name: &str);
  /// Reload a resource. That function is called when external stimuli create changes.
  ///
  /// # Failures
  ///
  /// If reloading the resource failed, `resource` is left untouched
  fn reload(manager: &mut Self::Manager, name: &str) -> Result<(), Self::Error>;
}

/// Check the presence of a key in a resource manager, and short-circuit the caller to return that
/// resource if force is false.
#[macro_export]
macro_rules! cache_fetch {
  ($manager:expr, $key:ident, $force:ident) => {
    if !$force {
      if let Some(x) = $manager.get($key) {
        return Ok(sync::Arc::downgrade(x));
      }
    }
  }
}

/// Default implementation to unload resources. It just removes the resource from its associated
/// manager, hence dropping it.
#[macro_export]
macro_rules! default_unload_impl {
  ($manager:expr, $key:ident) => {
    let _ = $manager.remove($key);
  }
}

// Outermost resource manager; i.e. gathers all resource managers.
pub struct ResourceManager {
  pub program_manager: ProgramManager
}

impl Default for ResourceManager {
  fn default() -> Self {
    ResourceManager {
      program_manager: Default::default()
    }
  }
}

impl ResourceManager {
  // Create a new resource manager.
  pub fn new() -> Self {
    ResourceManager::default()
  }
}
