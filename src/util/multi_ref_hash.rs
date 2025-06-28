use std::sync::{Arc, RwLock};

use crate::util::MutClone;

#[derive(Debug)]
pub struct MultiRefHash<T>(Arc<RwLock<T>>);
impl<T> Clone for MultiRefHash<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl<T> MutClone for MultiRefHash<T> {}
impl<T> PartialEq for MultiRefHash<T> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) // compara puntero, no contenido
  }
}
impl<T> MultiRefHash<T> {
  pub fn new(value: T) -> Self {
    Self(Arc::new(RwLock::new(value)))
  }
  pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
    self.0.read().unwrap()
  }

  pub fn write(&self) -> std::sync::RwLockWriteGuard<T> {
    self.0.write().unwrap()
  }
}
impl<T> std::hash::Hash for MultiRefHash<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    Arc::as_ptr(&self.0).hash(state); // usa la dirección del Rc para el hash
  }
}
impl<T> From<Arc<RwLock<T>>> for MultiRefHash<T> {
  fn from(value: Arc<RwLock<T>>) -> Self {
    Self(value)
  }
}
impl<T> From<T> for MultiRefHash<T> {
  fn from(value: T) -> Self {
    Self(Arc::new(RwLock::new(value)))
  }
}
impl<T> From<T> for MultiRefHash<Option<T>> {
  fn from(value: T) -> Self {
    Self(Arc::new(RwLock::new(Some(value))))
  }
}
impl<T> Eq for MultiRefHash<T> {}
impl<T> MultiRefHash<T>
where
  T: MutClone,
{
  pub fn cloned(&self) -> T {
    self.read().clone()
  }
}
impl<T> Default for MultiRefHash<T>
where
  T: Default,
{
  fn default() -> Self {
    Self(Default::default())
  }
}
impl<T> MultiRefHash<Option<T>> {
  pub fn on_ok<V>(&self, cb: impl FnOnce(&T) -> Option<V>) -> Option<V> {
    match self.read().as_ref() {
      Some(t) => cb(t),
      None => None,
    }
  }
  pub fn map<V>(&self, cb: impl FnOnce(&T) -> V) -> Option<V> {
   self.read().as_ref().map(cb)
  }
}
impl<T> MultiRefHash<Option<T>>
where
  T: MutClone,
{
  pub fn unwrap(&self) -> T {
    self.cloned().unwrap()
  }
}
impl MultiRefHash<usize> {
  pub fn saturating_sub(&self, rhs: usize) -> usize {
    self.read().saturating_sub(rhs)
  }
}
impl<T> MultiRefHash<Vec<T>> {
  pub fn push(&self, value: T) {
    self.write().push(value);
  }
  pub fn pop(&self) -> T {
    self.write().pop().unwrap()
  }
  pub fn clear(&self) {
    self.write().clear();
  }
  pub fn len(&self) -> usize {
    self.read().len()
  }
  pub fn is_empty(&self) -> bool {
    self.read().is_empty()
  }
}
impl<T> MultiRefHash<Vec<T>>
where
  T: MutClone,
{
  pub fn get(&self, index: usize) -> T {
    self.read().get(index).unwrap().clone()
  }
  pub fn last(&self) -> T {
    self.read().last().unwrap().clone()
  }
}
