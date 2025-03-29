use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use crate::runtime::values::{self, DefaultRefAgalValue};
mod fs;
mod math;
mod print;
mod time;
mod net;

type EvalResult = Option<values::DefaultRefAgalValue>;

pub const PREFIX_NATIVE_MODULES: &str = ":";

#[derive(Clone, Debug)]
struct Modules(Arc<RwLock<HashMap<String, DefaultRefAgalValue>>>);
impl Modules {
  fn has(&self, key: &str) -> bool {
    self.0.read().unwrap().contains_key(key)
  }
  fn get(&self, key: &str) -> EvalResult {
    let v = self.0.read().unwrap();
    v.get(key).cloned()
  }
  fn add(&self, key: &str, value: DefaultRefAgalValue) -> DefaultRefAgalValue {
    if self.has(key) {
      return self.get(key).unwrap_or_else(|| value);
    }
    self.0.write().unwrap().insert(key.to_string(), value.clone());
    value
  }
  fn as_ref(self) -> RefModules {
    RefModules(Arc::new(RwLock::new(self)))
  }
}

#[derive(Clone, Debug)]
pub struct RefModules(Arc<RwLock<Modules>>);
impl RefModules {
  pub fn new() -> Self {
    RefModules(Arc::new(RwLock::new(Modules(Arc::new(RwLock::new(
      HashMap::new(),
    ))))))
  }
  pub fn get_module(&self, key: &str) -> EvalResult {
    get_module(key, self.clone())
  }
  pub fn try_get(&self, key: &str) -> EvalResult {
    match self.0.read().unwrap().get(key) {
      Some(value) => Some(value.clone()),
      None => None,
    }
  }
  pub fn has(&self, key: &str) -> bool {
    self.0.read().unwrap().has(key)
  }
  fn get(&self, key: &str) -> DefaultRefAgalValue {
    self.0.read().unwrap().get(key).unwrap()
  }
  pub fn add(&self, key: &str, value: DefaultRefAgalValue) -> DefaultRefAgalValue {
    self.0.read().unwrap().add(key, value)
  }
}
pub fn get_module(key: &str, modules_manager: RefModules) -> EvalResult {
  if modules_manager.has(key) {
    let v = modules_manager.get(key);
    return Some(v);
  }
  if key == print::get_name(PREFIX_NATIVE_MODULES) {
    let value = print::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Some(value);
  }
  if key == fs::get_name(PREFIX_NATIVE_MODULES) {
    let value = fs::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Some(value);
  }
  if key == time::get_name(PREFIX_NATIVE_MODULES) {
    let value = time::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Some(value);
  }
  if key == math::get_name(PREFIX_NATIVE_MODULES) {
    let value = math::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Some(value);
  }
  if key == net::get_name(PREFIX_NATIVE_MODULES) {
    let value = net::get_module(PREFIX_NATIVE_MODULES);
    modules_manager.add(key, value.clone());
    return Some(value);
  }
  try_get_module(key, modules_manager)
}

mod proto;
fn try_get_module(key: &str, modules_manager: RefModules) -> EvalResult {
  if key.ends_with('/') {
    return None;
  }
  let path_parts: Vec<&str> = key.splitn(2, '/').collect();
  let module_name = *path_parts.get(0)?;
  let submodule_key = match path_parts.get(1) {
    Some(key) => *key,
    None => "",
  };
  if proto::get_name(PREFIX_NATIVE_MODULES) == module_name {
    return Some(proto::get_dir_module(
      PREFIX_NATIVE_MODULES,
      submodule_key,
      modules_manager,
    ));
  }
  None
}
