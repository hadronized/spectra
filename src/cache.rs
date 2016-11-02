use notify::{self, RecommendedWatcher, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use time::precise_time_s;

use id::Id;
use model::Model;
use resource::Load;

type Timestamp = f64;

/// Time to await after a resource update to establish that it should be reloaded.
const UPDATE_AWAIT_TIME: Timestamp = 0.1; // 100ms

pub struct CacheBlock<'a, T> where T: 'a {
  data: Vec<(T, PathBuf, (Receiver<Timestamp>, f64))>,
  ids: HashMap<String, Id<'a, T>>,
}

impl<'a, T> CacheBlock<'a, T> {
  pub fn new() -> Self {
    CacheBlock {
      data: Vec::new(),
      ids: HashMap::new(),
    }
  }
}

macro_rules! cache_struct {
  ($l:tt, $($n:ident: $t:ty),*) => {
    pub struct Cache<$l> {
      senders: Arc<Mutex<HashMap<PathBuf, Sender<Timestamp>>>>,
      $(
        $n: CacheBlock<$l, $t>
      ),*
    }

    impl<$l> Cache<$l> {
      pub fn new<P>(root:P) -> Self where P: AsRef<Path> {
        let senders: Arc<Mutex<HashMap<PathBuf, Sender<Timestamp>>>> = Arc::new(Mutex::new(HashMap::new()));

        // start watcher thread
        {
          let senders = senders.clone();
          let root = root.as_ref().to_path_buf();
          let (wsx, wrx) = channel();
          let mut watcher: RecommendedWatcher = Watcher::new(wsx).unwrap();

          let _ = thread::spawn(move || {
            let _ = watcher.watch(root);

            for event in wrx.iter() {
              match event {
                notify::Event { path: Some(path), op: Ok(notify::op::WRITE) } => {
                  if let Some(sx) = senders.lock().unwrap().get(&path) {
                    sx.send(precise_time_s()).unwrap();
                  }
                },
                _ => {}
              }
            }
          });
        }

        Cache {
          senders: senders,
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
                    let path_buf = path.to_owned();

                    // create the id if we have loaded the resource
                    let id: Id<$t> = (self.$n.data.len() as u32).into();

                    // create a channel to notify any update later and register the sender for the
                    // given path
                    let (sx, rx) = channel();
                    {
                      let mut senders = self.senders.lock().unwrap();
                      senders.insert(path_buf.clone(), sx);
                    }

                    // add the resource to the list of loaded ones
                    self.$n.data.push((resource, path_buf.clone(), (rx, precise_time_s())));
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

        fn get_by_id(&mut self, id: Self::Id) -> Option<&$t> {
          // synchronization
          if let Some(data) = self.$n.data.get_mut(*id as usize) {
            match (data.2).0.try_recv() {
              Ok(timestamp) if timestamp - (data.2).1 >= UPDATE_AWAIT_TIME => {
                // reload
                match <$t as Load>::load(&data.1) {
                  Ok(new_resource) => {
                    // replace the current resource with the freshly loaded one
                    data.0 = new_resource;
                  },
                  Err(e) => {
                    warn!("reloading resource from {:?} has failed: {:?}", data.1, e);
                  }
                }
              },
              _ => {}
            }
          } else {
            return None;
          }

          self.$n.data.get(*id as usize).map(|r| &r.0)
        }
      }
    )*
  }
}

pub trait Get<T> where T: Load {
  type Id;

  fn get_id(&mut self, name: &str) -> Option<Self::Id>;
  fn get_by_id(&mut self, id: Self::Id) -> Option<&T>;
  fn get(&mut self, name: &str) -> Option<&T> {
    self.get_id(name).and_then(move |i| self.get_by_id(i))
  }
}

cache_struct!('a,
              models: Model);
