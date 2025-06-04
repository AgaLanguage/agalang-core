use std::collections::{HashMap, HashSet};

use crate::{bytecode::vm::Thread, functions_names::CONSTRUCTOR};

use super::{MultiRefHash, Value};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Instance {
  name: String,
  extend: MultiRefHash<Option<Instance>>,
  poperties: MultiRefHash<HashMap<String, Value>>,
  public_properties: MultiRefHash<HashSet<String>>,
}

impl Instance {
  pub fn new(name: String) -> Self {
    Self {
      name,
      extend: None.into(),
      poperties: HashMap::new().into(),
      public_properties: HashSet::new().into(),
    }
  }
  pub fn get_instance_property(&self, key: &str, thread: &Thread) -> Option<Value> {
    // Si estamos dentro del metodo de clase o la propiedad es publica tenemos acceso
    let access = thread.get_calls().last().unwrap().in_class() || self.public_properties.borrow().contains(key);
    if access {
      self.poperties.borrow().get(key).cloned()
    } else {
      None
    }
  }
  pub fn set_instance_property(&self, key: &str, value: Value) -> Option<Value> {
    if self.poperties.borrow().contains_key(key) {
      return None;
    }
    Some(
      self
        .poperties
        .borrow_mut()
        .insert(key.to_string(), value.clone())
        .unwrap_or_default(),
    )
  }
  pub fn set_public_property(&self, key: &str, is_public: bool) {
    let current_is_public = self.public_properties.borrow().contains(key);
    if current_is_public == is_public {
      return;
    }
    if is_public {
      self.public_properties.borrow_mut().insert(key.to_string());
    } else {
      self.public_properties.borrow_mut().remove(key);
    }
  }
  pub fn _is_instance(&self, value: &Value) -> bool {
    if value.is_object() {
      let (_, instance) = value._as_map();
      let mut mut_instance = match instance {
        Some(ref_instance) => Some(ref_instance.borrow().clone()),
        None => None,
      };
      loop {
        match mut_instance {
          Some(instance) => {
            if instance.eq(self) {
              return true;
            }
            mut_instance = match instance.extend.borrow().clone() {
              Some(instance) => Some(instance),
              None => None,
            };
          }
          None => break,
        }
      }
    }
    false
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Class {
  name: String,
  extend: MultiRefHash<Option<Class>>,
  instance: MultiRefHash<Instance>,
  poperties: MultiRefHash<HashMap<String, Value>>,
}

impl Class {
  pub fn new(name: String) -> (MultiRefHash<Self>, MultiRefHash<Instance>) {
    let instance: MultiRefHash<Instance> = Instance::new(name.clone()).into();
    let class: MultiRefHash<Self> = Self {
      name,
      extend: None.into(),
      instance: instance.clone(),
      poperties: HashMap::new().into(),
    }
    .into();
    instance.borrow().set_instance_property(
      CONSTRUCTOR,
      Value::Ref(Value::Object(super::Object::Class(class.clone())).into()),
    );
    (class, instance)
  }
  pub fn set_instance_property(&self, key: &str, value: Value) -> Option<Value> {
    if self.poperties.borrow().contains_key(key) {
      return None;
    }
    Some(
      self
        .poperties
        .borrow_mut()
        .insert(key.to_string(), value.clone())
        .unwrap_or_default(),
    )
  }
  pub fn get_instance_property(&self, key: &str) -> Option<Value> {
    self.poperties.borrow().get(key).cloned()
  }
  pub fn get_instance(&self) -> Value {
    Value::Object(super::Object::Map(
      HashMap::new().into(),
      Some(self.instance.clone()),
    ))
  }
}
