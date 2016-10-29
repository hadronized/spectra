use std::marker::PhantomData;
use std::ops::Deref;

/// A typed identifier.
#[derive(Debug)]
pub struct Id<'a, T> where T: 'a {
  pub id: u32,
  _t: PhantomData<&'a T>
}

impl<'a, T> Id<'a, T> {
  pub fn new(id: u32) -> Self {
    Id {
      id: id,
      _t: PhantomData
    }
  }
}

impl<'a, T> Clone for Id<'a, T> {
  fn clone(&self) -> Self {
    self.id.into()
  }
}

impl<'a, T> Deref for Id<'a, T> {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.id
  }
}

impl<'a, T> From<u32> for Id<'a, T> {
  fn from(id: u32) -> Self {
    Id::new(id)
  }
}

