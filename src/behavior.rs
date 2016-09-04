pub type Time = f32;

pub struct Behavior<'a, A> {
  act: Box<Fn(Time) -> A + 'a>,
  switch: Box<Fn(Time) -> Option<Behavior<'a, A>> + 'a>
}

impl<'a, A> Behavior<'a, A> {
  pub fn new<F>(act: F) -> Self where F: Fn(Time) -> A + 'a {
    Behavior {
      act: Box::new(act),
      switch: Box::new(|_| None)
    }
  }

  pub fn switch<F>(self, switch: F) -> Self where F: Fn(Time) -> Option<Behavior<'a, A>> + 'a {
    Behavior {
      switch: Box::new(switch),
      .. self
    }
  }

  pub fn run(self, t: Time) -> (A, Behavior<'a, A>) {
    let r = (self.act)(t);
    let next = (self.switch)(t);

    match next {
      Some(next) => (r, next),
      None => (r, self)
    }
  }
}
