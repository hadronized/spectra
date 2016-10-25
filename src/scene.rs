use std::collections::HashMap;
use std::path::Path;

use model::Model;
use entity::Entity;
use resource::ResourceManager;
use transform::Transform;

pub type Id = u32;

/// The scene type.
///
/// This type gathers all the required objects a scene needs to correctly handle and render all
/// visual effects.
pub struct Scene<'a> {
  /// Resource manager; used to handle scarce resources.
  res_manager: ResourceManager,
  /// List of all models used in the scene.
  models: Vec<Model<'a>>,
  /// Model cache used to resolve Id based on instance name.
  model_cache: HashMap<String, Id>,
}

impl<'a> Scene<'a> {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      res_manager: ResourceManager::new(root),
      models: Vec::new(),
      model_cache: HashMap::new(),
    }
  }

  pub fn get_id<T>(&mut self, name: &str) -> Option<Id> where T: GetId {
    T::get_id(self, name)
  }

  pub fn get_model(&mut self, id

  pub fn resource_manager(&mut self) -> &mut ResourceManager {
    &mut self.res_manager
  }
}

pub trait GetId {
  fn get_id<'a>(scene: &mut Scene<'a>, name: &str) -> Option<Id>;
}

impl<'b> GetId for Model<'b> {
  fn get_id<'a>(scene: &mut Scene<'a>, name: &str) -> Option<Id> {
    match scene.model_cache.get(name).cloned() {
      id@Some(..) => id,
      None => {
        // cache miss; load then
        let path_str = format!("data/models/{}.obj", name);
        let path = Path::new(&path_str);

        if path.exists() {
          match Model::load(&mut scene.res_manager, path) {
            Ok(model) => {
              let model_id = scene.models.len() as u32;

              // add the model to the list of loaded models
              scene.models.push(model);
              // add the model to the cache
              scene.model_cache.insert(name.to_owned(), model_id);

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
}
