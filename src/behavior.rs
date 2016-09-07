pub type Time = f32;

pub struct Behavior<'a, A> where A: 'a {
  closure: Box<Fn(Time) -> A + 'a>,
}

impl<'a, A> Behavior<'a, A> {
  pub fn new<F>(act: F) -> Self where F: Fn(Time) -> A + 'a {
    Behavior {
      closure: Box::new(act),
    }
  }

  pub fn run(&self, t: Time) -> A {
    (self.closure)(t)
  }
}

pub struct Node<'a, A> where A: 'a {
  behavior: &'a Behavior<'a, A>,
  next: Option<(Time, &'a Node<'a, A>)>
}

impl<'a, A> Node<'a, A> {
  pub fn new(behavior: &'a Behavior<'a, A>, next: Option<(Time, &'a Self)>) -> Self {
    Node {
      behavior: behavior,
      next: next
    }
  }

  pub fn run(&'a self, t: Time) -> (A, &'a Self) {
    let next = self.next;

    match next {
      Some((t_node, node)) if t >= t_node => node.run(t),
      _ => (self.behavior.run(t), self)
    }
  }
}
