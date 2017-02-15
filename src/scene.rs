use std::path::Path;
use std::rc::Rc;

use resource::{ResCache, Load, Reload, Res};

/// The scene type.
///
/// This type gathers all the required objects a scene needs to correctly handle and render all
/// visual effects.
pub struct Scene {
  /// Cache.
  pub cache: ResCache
}

impl Scene {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      cache: ResCache::new(root),
    }
  }

  pub fn get<T>(&mut self, key: &str, args: T::Args) -> Option<Res<T>> where T: Reload {
    self.cache.get(key, args)
  }
}
