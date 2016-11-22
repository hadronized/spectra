extern crate clap;

use clap::{Arg, App};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;

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
      println!("--> {:?}", resource.0);
      copy_file(resource);
    }
  }
}

fn copy_file(entry: &(PathBuf, &'static [u8])) {
  let path = entry.0.as_path();
  let parent = path.parent().unwrap_or(&Path::new("."));

  create_dir_all(parent).unwrap();

  if let Ok(mut file) = File::create(path) {
    file.write_all(entry.1).unwrap();
  }
}
