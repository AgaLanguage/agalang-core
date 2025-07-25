use std::collections::{HashMap, HashSet};

use crate::{
  functions_names::{CONSTRUCTOR, SUPER},
  util::MutClone,
};

use super::{MultiRefHash, Value};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Instance {
  name: String,
  extend: MultiRefHash<Option<Instance>>,
  poperties: MultiRefHash<HashMap<String, Value>>,
  public_properties: MultiRefHash<HashSet<String>>,
}
impl MutClone for Instance {}

impl Instance {
  pub fn new(name: String) -> Self {
    let this = Self {
      name,
      extend: None.into(),
      poperties: Default::default(),
      public_properties: Default::default(),
    };
    this.set_instance_property(SUPER, Default::default(), true);
    this.set_instance_property(CONSTRUCTOR, Value::Ref(Default::default()), true);
    this
  }
  pub fn get_type(&self) -> &str {
    &self.name
  }
  pub fn get_instance_property(
    &self,
    key: &str,
    thread: &crate::interpreter::Thread,
  ) -> Option<Value> {
    if !self.poperties.read().contains_key(key) {
      return self
        .extend
        .on_ok(|extend| extend.get_instance_property(key, thread));
    }
    let access = if self.public_properties.read().contains(key) {
      true
    } else if let Some(class) = thread.get_calls().last().unwrap().in_class() {
      self
        .clone()
        .is_instance(class.read().instance.read().as_ref().unwrap().clone())
    } else {
      false
    };
    if access {
      self.poperties.read().get(key).cloned()
    } else {
      None
    }
  }
  pub fn set_instance_property(&self, key: &str, value: Value, is_public: bool) -> Value {
    if !self.poperties.read().contains_key(key) {
      self.set_public_property(key, is_public);
    }
    self
      .poperties
      .write()
      .insert(key.to_string(), value.clone())
      .unwrap_or_default()
  }
  fn set_public_property(&self, key: &str, is_public: bool) {
    let current_is_public = self.public_properties.read().contains(key);
    if current_is_public == is_public {
      return;
    }
    if is_public {
      self.public_properties.write().insert(key.to_string());
    } else {
      self.public_properties.write().remove(key);
    }
  }
  pub fn _is_instance_of(&self, value: &Value) -> bool {
    let (_, instance) = value.as_map();
    match instance.cloned() {
      Some(t) => self.is_instance(t),
      None => false,
    }
  }
  pub fn is_instance(&self, instance: Instance) -> bool {
    let mut mut_instance = Some(instance.clone());
    loop {
      match mut_instance {
        Some(instance) => {
          if instance.eq(self) {
            return true;
          }
          mut_instance = instance.extend.map(|instance| instance.clone());
        }
        None => return false,
      }
    }
  }
  pub fn ovwerwrite_instance_property(&self, key: &str, value: Value) {
    self
      .poperties
      .write()
      .insert(key.to_string(), value.clone());
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Class {
  name: String,
  extend: MultiRefHash<Option<Class>>,
  instance: MultiRefHash<Option<Instance>>,
  poperties: MultiRefHash<HashMap<String, Value>>,
}

impl Class {
  pub fn new(name: String) -> MultiRefHash<Self> {
    let instance: MultiRefHash<Option<Instance>> = Some(Instance::new(name.clone())).into();
    let class: MultiRefHash<Self> = Self {
      name,
      extend: None.into(),
      instance: instance.clone(),
      poperties: Default::default(),
    }
    .into();
    instance.map(|this| {
      this.ovwerwrite_instance_property(
        CONSTRUCTOR,
        Value::Ref(Value::Object(super::Object::Class(class.clone())).into()),
      );
    });
    class
  }
  pub fn get_type(&self) -> &str {
    &self.name
  }
  pub fn has_parent(&self) -> bool {
    self.extend.read().is_some()
  }
  pub fn get_parent(&self) -> MultiRefHash<Option<Class>> {
    self.extend.clone()
  }
  pub fn set_parent(&self, parent: Class) {
    *self.instance.read().as_ref().unwrap().extend.write() = parent.instance.cloned();
    *self.extend.write() = Some(parent);
  }
  pub fn set_instance_property(&self, key: &str, value: Value) -> Value {
    self
      .poperties
      .write()
      .insert(key.to_string(), value.clone())
      .unwrap_or_default()
  }
  pub fn get_instance_property(&self, key: &str) -> Option<Value> {
    self.poperties.read().get(key).cloned()
  }
  pub fn make_instance(&self) -> Value {
    self
      .extend
      .map(|parent| {
        let parent_instance = parent.make_instance();
        let (obj, instance) = parent_instance.as_map();
        let parent_instance = Value::Object(super::Object::Map(obj.clone(), instance));
        self
          .instance
          .map(|instance| instance.ovwerwrite_instance_property(SUPER, parent_instance));
        Value::Object(super::Object::Map(obj, self.instance.clone()))
      })
      .unwrap_or_else(|| {
        Value::Object(super::Object::Map(
          Default::default(),
          self.instance.clone(),
        ))
      })
  }
  pub fn get_instance(&self) -> MultiRefHash<Option<Instance>> {
    self.instance.clone()
  }
  pub fn is_instance(&self, instance: &Instance) -> bool {
    self.instance.map(|i| i == instance).unwrap_or_default()
  }
}

impl MutClone for Class {}
