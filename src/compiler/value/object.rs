use std::{collections::HashMap, hash::Hash};

use crate::{functions_names::STRING, MultiRefHash, StructTag};

use super::{Class, Function, Instance, Value};

pub const MAP_TYPE: &str = "objeto";
pub const LIST_TYPE: &str = "lista";
pub const CLASS_TYPE: &str = "clase";

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Object {
  Map(
    MultiRefHash<HashMap<String, Value>>,
    MultiRefHash<Option<Instance>>,
  ),
  //Set(MultiRefHash<HashSet<Value>>),
  Array(MultiRefHash<Vec<Value>>),
  Function(MultiRefHash<Function>),
  Class(MultiRefHash<Class>),
}
impl Object {
  pub fn get_type(&self) -> &str {
    match self {
      Self::Function(f) => f.read().get_type(),
      Self::Map(_, _) => MAP_TYPE,
      Self::Array(_) => LIST_TYPE,
      Self::Class(_) => CLASS_TYPE,
      //Self::Set(_) => "conjunto",
    }
  }

  pub fn set_object_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      // se usa Some(option.unwrap_or_default()) para que se envie por defecto un valor vacio en vez de un error con None
      Self::Map(obj, _) => {
        obj.write().insert(key.to_string(), value.clone());
        Some(value)
      }
      Self::Array(array) => {
        let index = key.parse::<usize>().ok()?;
        let mut vec = array.write();
        if index >= vec.len() {
          vec.resize(index + 1, Value::Never);
        }
        vec[index] = value.clone();
        Some(value)
      }
      _ => None,
    }
  }

  pub fn get_object_property(&self, key: &str) -> Option<Value> {
    match self {
      // se usa Some(option.unwrap_or_default()) para que se envie por defecto un valor vacio en vez de un error con None
      Self::Map(obj, _) => Some(obj.read().get(key).cloned().unwrap_or_default()),
      Self::Array(array) => Some(
        array
          .read()
          .get(key.parse::<usize>().ok()?)
          .cloned()
          .unwrap_or_default(),
      ),
      _ => None,
    }
  }

  pub fn get_instance_property(
    &self,
    key: &str,
    thread: &crate::interpreter::Thread,
  ) -> Option<Value> {
    match (self, key) {
      (Self::Map(_, instance), key) => instance.on_ok(|t| t.get_instance_property(key, thread)),
      (Self::Class(class), key) => class.read().get_instance_property(key),
      (Self::Array(array), "longitud") => Some(Value::Number(array.read().len().into())),
      (value, key) => {
        let proto_cache = thread
          .get_async()
          .read()
          .get_module()
          .read()
          .get_vm()
          .read()
          .cache
          .proto
          .clone();
        crate::interpreter::proto::proto(value.get_type().to_string(), proto_cache.clone())?
          .get_instance_property(key, thread)
      }
    }
  }
  pub fn as_string(&self, thread: &crate::interpreter::Thread) -> String {
    match self {
      Self::Map(_, i) => i
        .on_ok(|t| t.get_instance_property(STRING, thread))
        .map(|t|t.as_string(thread)),
      _ => None,
    }.unwrap_or_else(||self.to_string())
  }
}
impl From<Function> for Object {
  fn from(value: Function) -> Self {
    Self::Function(value.into())
  }
}
impl From<HashMap<String, Value>> for Object {
  fn from(value: HashMap<String, Value>) -> Self {
    Self::Map(value.into(), None.into())
  }
}
impl From<Vec<Value>> for Object {
  fn from(value: Vec<Value>) -> Self {
    Self::Array(value.into())
  }
}
impl From<Vec<u8>> for Object {
  fn from(value: Vec<u8>) -> Self {
    Self::Array(
      value
        .iter()
        .map(|b| Value::Byte(*b))
        .collect::<Vec<_>>()
        .into(),
    )
  }
}
impl From<&str> for Object {
  fn from(value: &str) -> Self {
    Object::Array(
      value
        .chars()
        .map(|c| Value::Char(c))
        .collect::<Vec<_>>()
        .into(),
    )
  }
}
impl ToString for Object {
  fn to_string(&self) -> String {
    match self {
      Self::Function(f) => f.read().to_string(),
      Self::Map(_, i) => i
        .map(|t| format!("<Instancia {}>", t.get_type()))
        .unwrap_or("<objeto>".to_string()),
      Self::Class(c) => {
        let has_parent = c.read().has_parent();
        let class = c.read().get_type().to_string();
        format!(
          "<clase {}>",
          if has_parent {
            let c_ref = c.read();
            let parent = c_ref.get_parent();

            let parent_ref = parent.read();
            let v = parent_ref.as_ref().unwrap().get_type();
            format!("{class} ext {v}")
          } else {
            class
          }
        )
      }
      _ => format!("<{}>", self.get_type()),
    }
  }
}
impl std::fmt::Debug for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
impl crate::Encode for Object {
  fn encode(&self) -> Result<Vec<u8>, String> {
    match self {
      Object::Map(_, _) => Ok(vec![StructTag::Map as u8]),
      Object::Function(function) => function.read().encode(),
      Object::Array(_) => Ok(vec![StructTag::Array as u8]),
      Object::Class(c) => {
        let mut encode = vec![StructTag::Class as u8];
        encode.extend(c.read().get_type().to_string().encode()?);
        Ok(encode)
      }
    }
  }
}
