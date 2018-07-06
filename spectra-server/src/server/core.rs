use std::sync::mpsc::{Receiver, Sender, channel};

use msg::Msg;

pub trait Server {
  fn spawn(self, sx: Sender<Msg>);
}

pub fn start_server<S>(s: S) -> Receiver<Msg> where S: Server {
  let (sx, rx) = channel();
  s.spawn(sx);
  rx
}
