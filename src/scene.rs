use std::collections::HashMap;
use std::path::Path;

use anim::Cont;
use id::Id;
use model::Model;
use resource::ResourceManager;

/// The scene type.
///
/// This type gathers all the required objects a scene needs to correctly handle and render all
/// visual effects.
pub struct Scene<'a> {
  /// Resource manager; used to handle scarce resources.
  res_manager: ResourceManager,
  /// All models used in the scene.
  models: Vec<Model>,
  /// Model cache used to resolve Id based on instance name.
  model_cache: HashMap<String, Id<'a, Model>>,
}

impl<'a> Scene<'a> {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      res_manager: ResourceManager::new(root),
      models: Vec::new(),
      model_cache: HashMap::new(),
    }
  }

  pub fn resource_manager(&mut self) -> &mut ResourceManager {
    &mut self.res_manager
  }
}

pub trait Get<T>: Sized {
  type Id;

  fn get_id(&mut self, name: &str) -> Option<Self::Id>;
  fn get_by_id(&self, id: Self::Id) -> Option<&T>;
  fn get(&mut self, name: &str) -> Option<&T> {
    self.get_id(name).and_then(move |i| self.get_by_id(i))
  }
}

impl<'a> Get<Model> for Scene<'a> {
  type Id = Id<'a, Model>;

  fn get_id(&mut self, name: &str) -> Option<Self::Id> {
    match self.model_cache.get(name).cloned() {
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
          match Model::load(&mut self.res_manager, path) {
            Ok(model) => {
              let model_id: Id<Model> = (self.models.len() as u32).into();

              // add the model to the list of loaded models
              self.models.push(model);
              // add the model to the cache
              self.model_cache.insert(name.to_owned(), model_id.clone());

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

  fn get_by_id(&self, id: Self::Id) -> Option<&Model> {
    self.models.get(*id as usize)
  }
}
