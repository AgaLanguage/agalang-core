

use crate::compiler::Value;

#[derive(Debug, Clone, Default)]
pub struct DataManager<K: Eq + std::hash::Hash, V: Clone> {
  data: crate::MultiRefHash<std::collections::HashMap<K, V>>,
}
impl<K: Eq + std::hash::Hash, V: Clone> DataManager<K, V> {
  pub fn get(&self, key: &K) -> V {
    self.data.read().get(key).cloned().unwrap()
  }
  pub fn set(&mut self, key: K, value: V) {
    self.data.write().insert(key, value);
  }
  pub fn has(&self, key: &K) -> bool {
    self.data.read().contains_key(key)
  }
}

pub type DataCache = DataManager<String, Value>;

#[derive(Clone, Debug, Default)]
pub struct Cache {
  pub proto: DataCache,
  pub libs: DataCache,
}
