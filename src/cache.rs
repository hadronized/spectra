use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;

use id::Id;
use model::Model;
use object::Object;
use resource::Load;

pub struct CacheBlock<'a, T> where T: 'a {
  data: Vec<T>,
  ids: HashMap<String, Id<'a, T>>
}

impl<'a, T> CacheBlock<'a, T> {
  pub fn new() -> Self {
    CacheBlock {
      data: Vec::new(),
      ids: HashMap::new()
    }
  }
}

macro_rules! cache_struct {
  ($l:tt, $($n:ident: $t:ty),*) => {
    pub struct Cache<$l> {
      $(
        $n: CacheBlock<$l, $t>
      ),*
    }

    impl<$l> Cache<$l> {
      pub fn new() -> Self {
        Cache {
          $(
            $n: CacheBlock::new()
          ),*
        }
      }
    }

    $(
      impl<$l> Get<$t> for Cache<$l> {
        type Id = Id<$l, $t>;

        fn get_id(&mut self, name: &str) -> Option<Self::Id> {
          let path_str = format!("data/{}/{}", stringify!($n), name);
          let path = Path::new(&path_str);

          match self.$n.ids.get(name).cloned() {
            id@Some(..) => {
              deb!("cache hit for {}", path_str);
              id
            },
            None => {
              deb!("cache miss for {}", path_str);

              // specific loading
              if path.exists() {
                match <$t as Load>::load(&path) {
                  Ok(resource) => {
                    // create the id if we have loaded the resource
                    let id: Id<$t> = (self.$n.data.len() as u32).into();

                    // add the resource to the list of loaded ones
                    self.$n.data.push(resource);
                    // cache the resource
                    self.$n.ids.insert(name.to_owned(), id.clone());

                    Some(id)
                  },
                  Err(e) => {
                    err!("unable to load resource from {}: {:?}", path_str, e);
                    None
                  }
                }
              } else { // path doesn’t exist
                err!("ressource at {} cannot be found", path_str);
                None
              }
            }
          }
        }

        fn get_by_id(&self, id: Self::Id) -> Option<&$t> {
          self.$n.data.get(*id as usize)
        }
      }
    )*
  }
}

pub trait Get<T> where T: Load {
  type Id;

  fn get_id(&mut self, name: &str) -> Option<Self::Id>;
  fn get_by_id(&self, id: Self::Id) -> Option<&T>;
  fn get(&mut self, name: &str) -> Option<&T> {
    self.get_id(name).and_then(move |i| self.get_by_id(i))
  }
}

cache_struct!('a,
              models: Model);
