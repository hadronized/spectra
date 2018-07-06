use serde_json::de::from_str;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::thread;

use msg::Msg;
use server::core::Server;

pub struct TcpServer;

impl Server for TcpServer {
  fn spawn(self, sx: Sender<Msg>) {
    thread::spawn(move || {
      let listener = TcpListener::bind("127.0.0.1:6666").unwrap(); 

      for stream in listener.incoming() {
        let stream = stream.unwrap();

        deb!("stream connected: {:?}", stream);

        for line in  BufReader::new(stream).lines() {
          let line = line.unwrap();

          match from_str(&line) {
            Ok(msg) => {
              deb!("received command: {:?}", msg);
              let _ = sx.send(msg);
            }

            Err(e) => err!("wrong command: {}", e)
          }
        }

        deb!("stream disconnected");
      }
    });
  }
}
