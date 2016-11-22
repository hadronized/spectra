extern crate clap;

use clap::{Arg, App};

include!(concat!(env!("OUT_DIR"), "/resources.rs"));

fn main() {
  let options = App::new("ionize")
    .arg(Arg::with_name("bootstrap")
         .short("b")
         .long("bootstrap")
         .takes_value(false)
         .help("bootstrap default resources into the current application"))
    .get_matches();

  if options.is_present("bootstrap") {
    println!("bootstraping resources");

    for resource in &get_resources() {
      println!("--> {:?}", resource);
    }
  }
}
