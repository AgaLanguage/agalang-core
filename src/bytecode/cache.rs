use std::collections::HashMap;

use super::value::{MultiRefHash, Value};

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

pub struct Cache {
  pub proto: DataCache,
  pub libs: DataCache,
}
impl Cache {
  pub fn new() -> Self {
    Self {
      proto: DataManager::new(),
      libs: DataManager::new(),
    }
  }
}