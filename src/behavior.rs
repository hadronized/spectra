/// Time used to synchronized the behaviors.
pub type Time = f32;

/// A behavior that might switch to another one.
///
/// A `Behavior` is a simple closure of time into `A`. Optionally, a `Behavior` might express the
/// need to switch to another behavior at runtime, based on the current time. That enables dynamic
/// switching of code with ease and a few boilerplate.
///
/// Keep in mind a `Behavior` is a closure that implements `Fn<Time>`.
pub struct Behavior<'a, A> where A: 'a {
  closure: Box<Fn(Time) -> A + 'a>,
  next: Option<(Time, &'a Behavior<'a, A>)>
}

impl<'a, A> Behavior<'a, A> {
  /// Create a new behavior that will behave as `act` until the optional next behavior is triggered.
  pub fn new<F>(act: F, next: Option<(Time, &'a Self)>) -> Self where F: Fn(Time) -> A + 'a {
    Behavior {
      closure: Box::new(act),
      next: next
    }
  }

  /// Run a behavior with the current time. The returned reference is the same reference as `self`
  /// if no switching has occurred. If it has, that reference represents the new switched behavior.
  ///
  /// There are two ways to use behaviors:
  ///
  ///   1. ignoring the returned reference ;
  ///   2. always updating the source reference with the returned one.
  ///
  /// The first way is very handy because of how switching is implemented – reference chaining. You
  /// can use the very first reference of a long behavior reference lists, you’ll always get the
  /// corresponding behavior for the given time. This is very handy especially when debugging, since
  /// you might jump in time.
  ///
  /// The second way is much efficient if you are using a time that goes forward until the end of
  /// the application – which should be the case for the release target. Because you always replace
  /// your reference with the *maybe*-switched one, you are sure that a switch will be always
  /// performed in *O(1)* complexity – while the first way is *O(N)*.
  pub fn run(&'a self, t: Time) -> (A, &'a Self) {
    let next = self.next;

    match next {
      Some((node_t, node)) if t >= node_t => node.run(t),
      _ => ((self.closure)(t), self)
    }
  }

  /// Create a new behavior that will switch to another based on the current time.
  ///
  /// This function discards the switching property of the input behavior.
  pub fn switch(&'a self, t: Time, sw: &'a Self) -> Self {
    Behavior {
      closure: Box::new(move |t| {
          (self.closure)(t)
        }),
      next: Some((t, sw))
    }
  }
}
