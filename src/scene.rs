use std::path::Path;
use std::rc::Rc;

use id::Id;
use resource::{Cache, Load, Get, Reload};

/// The scene type.
///
/// This type gathers all the required objects a scene needs to correctly handle and render all
/// visual effects.
pub struct Scene<'a> {
  /// Cache.
  pub cache: Cache<'a>
}

impl<'a> Scene<'a> {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      cache: Cache::new(root),
    }
  }

  pub fn get_id<T>(&mut self, name: &str, args: <T as Load<'a>>::Args) -> Option<Id<'a, T>> where Cache<'a>: Get<'a, T>, T: 'a + Reload<'a> {
    self.cache.get_id(name, args)
  }

  pub fn get_by_id<T>(&mut self, id: &Id<'a, T>) -> Option<Rc<T>> where Cache<'a>: Get<'a, T>, T: 'a + Reload<'a> {
    self.cache.get_by_id(id)
  }

  pub fn get<T>(&mut self, name: &str, args: <T as Load<'a>>::Args) -> Option<Rc<T>> where Cache<'a>: Get<'a, T>, T: 'a + Reload<'a> {
    self.cache.get(name, args)
  }
}

#[macro_export]
macro_rules! get_id {
  ($scene:ident, $name:expr) => {
    $scene.get_id($name, ())
  };

  ($scene:ident, $name:expr, $($arg:expr),*) => {
    $scene.get_id($name, ($($arg),*))
  }
}

#[macro_export]
macro_rules! get {
  ($scene:ident, $name:expr) => {
    $scene.get($name, ())
  };

  ($scene:ident, $name:expr, $($arg:expr),*) => {
    $scene.get($name, ($($arg),*))
  }
}
