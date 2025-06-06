use std::collections::HashMap;

use crate::compiler::{MultiRefHash, Value};

#[derive(Debug, Clone, Default)]
pub struct DataManager<K: Eq + std::hash::Hash, V: Clone>
{
  data: MultiRefHash<HashMap<K, V>>,
}
impl<K: Eq + std::hash::Hash, V: Clone> DataManager<K, V> {
  pub fn get(&self, key: &K) -> V {
    self.data.borrow().get(key).cloned().unwrap()
  }
  pub fn set(&mut self, key: K, value: V) {
    self.data.borrow_mut().insert(key, value);
  }
  pub fn has(&self, key: &K) -> bool {
    self.data.borrow().contains_key(key)
  }
}

pub type DataCache = DataManager<String, Value>;

#[derive(Clone, Debug, Default)]
pub struct Cache {
  pub proto: DataCache,
  pub libs: DataCache,
}