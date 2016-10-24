use std::collections::HashMap;
use std::path::Path;

use model::Model;
use resource::ResourceManager;

pub struct Scene<'a> {
  /// Resource manager.
  res_manager: ResourceManager,
  /// List of all models used in the scene.
  models: Vec<Model<'a>>,
  /// Model cache used to resolve Id based on instance name.
  model_cache: HashMap<String, Id>,
  /// Scene model instances.
  model_instances: Vec<Id>
}

impl<'a> Scene<'a> {
  pub fn new<P>(root: P) -> Self where P: AsRef<Path>{
    Scene {
      res_manager: ResourceManager::new(root),
      models: Vec::new(),
      model_cache: HashMap::new(),
      model_instances: Vec::new()
    }
  }

  pub fn get_id<T>(&mut self, inst_name: &str) -> Option<Id> where T: GetId {
    T::get_id(self, inst_name)
  }

  pub fn resource_manager(&mut self) -> &mut ResourceManager {
    &mut self.res_manager
  }
}

pub type Id = u32;

pub trait GetId {
  fn get_id<'a>(scene: &mut Scene<'a>, inst_name: &str) -> Option<Id>;
}

impl<'b> GetId for Model<'b> {
  fn get_id<'a>(scene: &mut Scene<'a>, inst_name: &str) -> Option<Id> {
    match scene.model_cache.get(inst_name).cloned() {
      id@Some(..) => id,
      None => {
        // cache miss; load then
        let path_str = format!("data/models/{}.obj", inst_name);
        let path = Path::new(&path_str);

        if path.exists() {
          match Model::load(&mut scene.res_manager, path) {
            Ok(model) => {
              let model_id = scene.models.len() as u32;

              // add the model to the list of loaded models
              scene.models.push(model);
              // add the model to the cache
              scene.model_cache.insert(inst_name.to_owned(), model_id);

              Some(model_id)
            },
            Err(e) => {
              err!("unable to load model '{}': '{:?}'", inst_name, e);
              None
            }
          }
        } else { // TODO: add a manifest override to look in somewhere else
          err!("model '{}' cannot be found", inst_name);
          None
        }
      }
    }
  }
}
