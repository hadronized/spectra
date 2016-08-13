/// A pulse in time, representing an event occurrence and something happening.
///
/// `T` is the type of time.
pub struct Pulse<'a, T> {
  /// Activation time threshold.
  pub time_threshold: T,
  /// Carried closure representing the pulse.
  action: Action<'a, T>
}

pub type Action<'a, T> = Box<Fn(T) + 'a>;

impl<'a, T> Pulse<'a, T> where T: PartialOrd {
  /// Create a new `Pulse` which carried action occurs at `t`.
  pub fn new<F>(time_threshold: T, action: F) -> Self where F: 'a + Fn(T) {
    Pulse {
      time_threshold: time_threshold,
      action: Box::new(action)
    }
  }

  /// Try to access the action of a `Pulse`. Returns a reference to it if it’s already occured.
  pub fn try_action(&self, t: T) -> Option<&Action<T>> {
    if t >= self.time_threshold {
      Some(&self.action)
    } else {
      None
    }
  }
}
