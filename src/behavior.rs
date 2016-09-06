pub type Time = f32;

pub struct Behavior<'a, A> where A: 'a {
  closure: Box<for <'b> Fn(Time, &'b Behavior<'a, A>) -> (A, &'b Behavior<'a, A>) + 'a>,
}

impl<'a, A> Behavior<'a, A> {
  pub fn new<F>(act: F) -> Self where F: Fn(Time) -> A + 'a {
    Behavior {
      closure: Box::new(move |t, b| {
        (act(t), b)
      })
    }
  }

  pub fn run(&self, t: Time) -> (A, &Self) {
    (self.closure)(t, self)
  }

  //pub fn switch<F>(self, sw: F) -> Self where F: for <'b> Fn(Time, &'b Self) -> Option<&'a Self> + 'a {
  //  Behavior {
  //    closure: Box::new(move |t, b| {
  //      match sw(t, b) {
  //        Some(sw_b) => sw_b.run(t),
  //        None => self.run(t)
  //      }
  //    })
  //  }
  //}
}
