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

pub type Id = u32;

pub trait GetId {
  fn get_id<'a>(scene: &mut Scene<'a>, inst_name: String) -> Option<Id>;
}

impl<'b> GetId for Model<'b> {
  fn get_id<'a>(scene: &mut Scene<'a>, inst_name: String) -> Option<Id> {
    match scene.model_cache.get(&inst_name).cloned() {
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
              scene.model_cache.insert(inst_name, model_id);

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
