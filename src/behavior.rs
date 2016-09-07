pub type Time = f32;

pub struct Behavior<'a, A> where A: 'a {
  closure: Box<Fn(Time) -> A + 'a>,
  next: Option<(Time, &'a Behavior<'a, A>)>
}

impl<'a, A> Behavior<'a, A> {
  pub fn new<F>(act: F, next: Option<(Time, &'a Self)>) -> Self where F: Fn(Time) -> A + 'a {
    Behavior {
      closure: Box::new(act),
      next: next
    }
  }

  pub fn run(&'a self, t: Time) -> (A, &'a Self) {
    let next = self.next;

    match next {
      Some((node_t, node)) if t >= node_t => node.run(t),
      _ => ((self.closure)(t), self)
    }
  }
}
