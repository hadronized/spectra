#[macro_use]
extern crate clap;

use clap::{App, AppSettings, SubCommand};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/resources.rs"));

fn main() {
  let options = App::new("spectra")
    .version(crate_version!())
    .author(crate_authors!())
    .about("Spectra engine utility")
    .setting(AppSettings::SubcommandRequired)
    .subcommand(SubCommand::with_name("bootstrap")
         .about("Create default resources in your end-user project"))
    .get_matches();

  if options.subcommand_matches("bootstrap").is_some() {
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
