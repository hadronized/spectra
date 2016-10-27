use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;

use model::Model;
use resource::ResourceManager;

/// A typed identifier.
#[derive(Debug)]
pub struct Id<T> {
  pub id: u32,
  _t: PhantomData<*const T>
}

impl<T> Id<T> {
  pub fn new(id: u32) -> Self {
    Id {
      id: id,
      _t: PhantomData
    }
  }
}

impl<T> Clone for Id<T> {
  fn clone(&self) -> Self {
    self.id.into()
  }
}

impl<T> Deref for Id<T> {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.id
  }
}

impl<T> From<u32> for Id<T> {
  fn from(id: u32) -> Self {
    Id::new(id)
  }
}

/// The scene type.
///
/// This type gathers all the required objects a scene needs to correctly handle and render all
/// visual effects.
pub struct Scene {
  /// Resource manager; used to handle scarce resources.
  res_manager: ResourceManager,
  /// List of all models used in the scene.
  models: Vec<Model>,
  /// Model cache used to resolve Id based on instance name.
  model_cache: HashMap<String, Id<Model>>,
}

impl Scene {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      res_manager: ResourceManager::new(root),
      models: Vec::new(),
      model_cache: HashMap::new(),
    }
  }

  pub fn get_id<T>(&mut self, name: &str) -> Option<Id<T>> where T: Get {
    T::get_id(self, name)
  }

  pub fn get<T>(&self, id: Id<T>) -> Option<&T> where T: Get {
    T::get(self, id)
  }

  pub fn resource_manager(&mut self) -> &mut ResourceManager {
    &mut self.res_manager
  }
}

pub trait Get: Sized {
  fn get_id(scene: &mut Scene, name: &str) -> Option<Id<Self>>;
  fn get(scene: &Scene, id: Id<Self>) -> Option<&Self>;
}

impl Get for Model {
  fn get_id(scene: &mut Scene, name: &str) -> Option<Id<Self>> {
    match scene.model_cache.get(name).cloned() {
      id@Some(..) => {
        deb!("cache hit for model \"{}\"", name);
        id
      },
      None => {
        // cache miss; load then
        let path_str = format!("data/models/{}.obj", name);
        let path = Path::new(&path_str);

        deb!("cache miss for model \"{}\"", name);

        if path.exists() {
          match Model::load(&mut scene.res_manager, path) {
            Ok(model) => {
              let model_id: Id<Model> = (scene.models.len() as u32).into();

              // add the model to the list of loaded models
              scene.models.push(model);
              // add the model to the cache
              scene.model_cache.insert(name.to_owned(), model_id.clone());

              Some(model_id)
            },
            Err(e) => {
              err!("unable to load model '{}': '{:?}'", name, e);
              None
            }
          }
        } else { // TODO: add a manifest override to look in somewhere else
          err!("model '{}' cannot be found", name);
          None
        }
      }
    }
  }

  fn get(scene: &Scene, id: Id<Self>) -> Option<&Self> {
    scene.models.get(*id as usize)
  }
}
