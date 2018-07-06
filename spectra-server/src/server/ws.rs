use serde_json::de::from_str;
use std::sync::mpsc::Sender;
use std::thread;
use ws::{self, listen};

use msg::Msg;
use server::core::Server;

pub struct WSServer;

impl Server for WSServer {
  fn spawn(self, sx: Sender<Msg>) {
    thread::spawn(move || {
      loop {
        let _ = listen("127.0.0.1:6666", |_| {
          deb!("stream connected");

          |input| {
            if let ws::Message::Text(line) = input {
              match from_str(&line) {
                Ok(msg) => {
                  deb!("received command: {:?}", msg);
                  let _ = sx.send(msg);
                }

                Err(e) => err!("wrong command: {}", e)
              }
            }

            Ok(())
          }
        });

        deb!("stream disconnected");
      }
    });
  }
}
