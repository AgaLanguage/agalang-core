use std::collections::HashMap;

use crate::value::MultiRefHash;

#[derive(Debug, Clone)]
pub struct DataManager<K: Eq + std::hash::Hash, V: Clone>
{
  data: MultiRefHash<HashMap<K, V>>,
}
impl<K: Eq + std::hash::Hash, V: Clone> DataManager<K, V> {
  pub fn new() -> Self {
    DataManager {
      data: HashMap::new().into(),
    }
  }
  pub fn get(&self, key: &K) -> Option<V> {
    self.data.borrow().get(key).cloned()
  }
  pub fn set(&mut self, key: K, value: V) {
    self.data.borrow_mut().insert(key, value);
  }
  pub fn has(&self, key: &K) -> bool {
    self.data.borrow().contains_key(key)
  }
}