use std::fmt;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// A typed identifier.
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

impl<'a, T> fmt::Debug for Id<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "Id {{ id: {} }}", self.id)
  }
}

impl<'a, T> Clone for Id<'a, T> {
  fn clone(&self) -> Self {
    self.id.into()
  }
}

impl<'a, T> PartialEq<Id<'a, T>> for Id<'a, T> {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<'a, T> Eq for Id<'a, T> {}

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

impl<'a, T> Hash for Id<'a, T> {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    self.id.hash(state)
  }
}
