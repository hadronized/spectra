#[macro_use]
extern crate spectra;

mod app_demo;
mod shadertoy;

use spectra::sys::time::Time;

use shadertoy::AnimatedParam;

struct D;

impl app_demo::Demo for D {
  fn render_frame(&mut self, t: Time)  {
  }
}

fn main() {
  app_demo::run(D);
}
