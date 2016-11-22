use std::env::var;
use std::io::{self, Write};
use std::fs::{DirEntry, File, read_dir};
use std::path::{Path, PathBuf};
use std::process::exit;

fn main() {
  let out_dir = var("OUT_DIR").unwrap();
  let path = Path::new(&out_dir).join("resources.rs");

  if path.exists() {
    exit(0);
  }

  let mut manifest = File::create(path).unwrap();
  let mut resources: Vec<PathBuf> = Vec::new();

  visit_dirs("ionize/data", &mut |entry| {
    resources.push(Path::new("../..").join(entry.path()));
  }).unwrap();

  write!(&mut manifest, "use std::path::PathBuf;\n").unwrap();
  write!(&mut manifest, "pub fn get_resources() -> Vec<(PathBuf, &'static [u8])> {{\n").unwrap();
  write!(&mut manifest, "  let mut resources = Vec::new();\n").unwrap();

  for entry in &resources {
    println!("cargo:warning=packing up resource {:?}", entry);
    write!(&mut manifest, "  resources.push((PathBuf::from(&{0:?}[13..]), include_bytes!({0:?}).as_ref()));\n", entry).unwrap();
  }

  write!(&mut manifest, "  resources\n").unwrap();
  write!(&mut manifest, "}}").unwrap(); // get_resources
}

fn visit_dirs<P, F>(dir: P, visitor: &mut F) -> io::Result<()> where P: AsRef<Path>, F: FnMut(&DirEntry) {
  let dir = dir.as_ref();

  if dir.is_dir() {
    for entry in read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() {
        visit_dirs(path, visitor).unwrap();
      } else {
        visitor(&entry);
      }
    }
  }

  Ok(())
}
