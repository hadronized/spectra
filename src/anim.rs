// FIXME: not sure we need mutability here, because it would lead into unreproductible effects
/// Continuous value.
///
/// This type wraps a `A` as a function of time `f32`. It has a simple semantic: `at`, giving the
/// value at the wished time.
pub struct Cont<'a, A> {
  closure: Box<FnMut(f32) -> A + 'a>
}

impl<'a, A> Cont<'a, A> {
  pub fn new<F>(f: F) -> Self where F: 'a + FnMut(f32) -> A {
    Cont {
      closure: Box::new(f)
    }
  }

  /// Turn a set of discret values that happen at given moments into a continuous step value.
  pub fn from_discrete(def: A, mut moments: Vec<(f32, A)>) -> Self where A: 'a + Clone {
    moments.sort_by(|_, b| b.0.partial_cmp(&b.0).unwrap());

    Cont {
      closure: Box::new(move |t| {
        moments.binary_search_by(|a| a.0.partial_cmp(&t).unwrap()).ok().map_or(def.clone(), |i| moments[i].1.clone())
      })
    }
  }

  pub fn at(&mut self, t: f32) -> A {
    (self.closure)(t)
  }
}
