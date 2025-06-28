use std::collections::{HashMap, HashSet};

use crate::{
  compiler::{Value, FALSE_NAME, NEVER_NAME, NULL_NAME, TRUE_NAME},
  MultiRefHash,
};

const THIS_NAME: &str = "esto";

const KEYWORDS: [&str; 5] = [FALSE_NAME, NULL_NAME, TRUE_NAME, NEVER_NAME, THIS_NAME];

#[derive(Debug, Clone)]
pub struct VarsManager {
  variables: HashMap<String, Value>,
  constants: HashSet<String>,
  link: Option<MultiRefHash<VarsManager>>,
}
impl VarsManager {
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      constants: HashSet::new(),
      link: None,
    }
  }
  pub fn get_global() -> Self {
    let mut this = Self::new();
    this.declare_keyword(NEVER_NAME, Value::Never);
    this.declare_keyword(NULL_NAME, Value::Null);
    this.declare_keyword(FALSE_NAME, Value::False);
    this.declare_keyword(TRUE_NAME, Value::True);
    this
  }
  pub fn crate_child(parent: MultiRefHash<Self>) -> Self {
    let mut this = Self::new();
    this.link = Some(parent);
    this
  }
  fn declare_keyword(&mut self, name: &str, value: Value) {
    self.variables.insert(name.to_string(), value.clone());
  }
  pub fn declare(&mut self, name: &str, value: Value, is_constant: bool) -> Option<Value> {
    if KEYWORDS.contains(&name) {
      return None;
    }
    if self.variables.contains_key(name) {
      return None;
    }
    if is_constant {
      self.constants.insert(name.to_string());
    }
    self.variables.insert(name.to_string(), value.clone());
    Some(value)
  }
  pub fn has(&self, name: &str) -> bool {
    self.variables.contains_key(name)
  }
  pub fn get(&self, name: &str) -> Option<&Value> {
    self.variables.get(name)
  }
  pub fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
    if !self.variables.contains_key(name)
      || self.constants.contains(name)
      || KEYWORDS.contains(&name)
    {
      return None;
    };
    self.variables.insert(name.to_string(), value.clone());
    Some(value)
  }
  pub fn set_this(mut self, this: Value) -> Self {
    self.declare_keyword(THIS_NAME, this);
    self
  }
  pub fn get_link(&self) -> Option<MultiRefHash<Self>> {
    self.link.clone()
  }
  pub fn remove(&mut self, name: &str) -> Option<Value> {
    if !self.variables.contains_key(name)
      || self.constants.contains(name)
      || KEYWORDS.contains(&name)
    {
      return None;
    }
    self.variables.remove(name)
  }
}
impl Default for VarsManager {
  fn default() -> Self {
    Self::new()
  }
}
