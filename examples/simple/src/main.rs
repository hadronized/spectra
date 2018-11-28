use spectra::app::demo::{Backbuffer, Builder, Demo, Key, Store, Time};
use spectra::app::runner::debug::Runner;
use spectra::resource::context::DefaultContext;

struct App {
}

impl Demo for App {
  type Context = DefaultContext;

  type Error = ();

  fn init(store: &mut Store<Self::Context, Key>, context: &mut Self::Context) -> Result<Self, Self::Error> {
    Err(())
  }

  fn resize(&mut self, context: &mut Self::Context, width: u32, height: u32) {
    unimplemented!();
  }

  fn render(&mut self, context: &mut Self::Context, t: Time, back_buffer: &Backbuffer, builder: Builder) {
    unimplemented!();
  }
}

fn main() {
  let result = Runner::run::<App>("simple example", 960, 540, DefaultContext::default());

  if let Err(e) = result {
    eprintln!("{}", e);
  }
}
