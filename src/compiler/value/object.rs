use std::{
  cell::RefCell,
  collections::HashMap,
  hash::{Hash, Hasher},
  rc::Rc,
};

use crate::StructTag;

use super::{Class, Function, Instance, Value};

#[derive(Debug, Clone)]
pub struct MultiRefHash<T>(Rc<RefCell<T>>);
impl<T> PartialEq for MultiRefHash<T> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0) // compara puntero, no contenido
  }
}
impl<T> MultiRefHash<T> {
  pub fn borrow(&self) -> std::cell::Ref<T> {
    self.0.borrow()
  }
  pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
    self.0.borrow_mut()
  }
}
impl<T> Hash for MultiRefHash<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Rc::as_ptr(&self.0).hash(state); // usa la dirección del Rc para el hash
  }
}
impl<T> From<Rc<RefCell<T>>> for MultiRefHash<T> {
  fn from(value: Rc<RefCell<T>>) -> Self {
    Self(value)
  }
}
impl<T> From<T> for MultiRefHash<T> {
  fn from(value: T) -> Self {
    Self(Rc::new(RefCell::new(value)))
  }
}
impl<T> From<T> for MultiRefHash<Option<T>> {
  fn from(value: T) -> Self {
    Self(Rc::new(RefCell::new(Some(value))))
  }
}
impl<T> Eq for MultiRefHash<T> {}
impl<T> MultiRefHash<T>
where
  T: Clone,
{
  pub fn cloned(&self) -> T {
    self.borrow().clone()
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
    match self.borrow().as_ref() {
      Some(t) => cb(t),
      None => None,
    }
  }
  pub fn on_some<V>(&self, cb: impl FnOnce(&T) -> V) -> Option<V> {
    match self.borrow().as_ref() {
      Some(t) => Some(cb(t)),
      None => None,
    }
  }
  pub fn as_ref(&self) -> Option<std::cell::Ref<T>> {
    let ref_opcion = self.borrow();
    if ref_opcion.is_some() {
      Some(std::cell::Ref::map(ref_opcion, |opt| opt.as_ref().unwrap()))
    } else {
      None
    }
  }
}

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
      Self::Function(f) => f.borrow().get_type(),
      Self::Map(_, _) => "objeto",
      Self::Array(_) => "lista",
      Self::Class(_) => "clase",
      //Self::Set(_) => "conjunto",
    }
  }

  pub fn set_object_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      // se usa Some(option.unwrap_or_default()) para que se envie por defecto un valor vacio en vez de un error con None
      Self::Map(obj, _) => {
        obj.borrow_mut().insert(key.to_string(), value.clone());
        Some(value)
      }
      Self::Array(array) => {
        let index = key.parse::<usize>().ok()?;
        let mut vec = array.borrow_mut();
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
      Self::Map(obj, _) => Some(obj.borrow().get(key).cloned().unwrap_or_default()),
      Self::Array(array) => Some(
        array
          .borrow()
          .get(key.parse::<usize>().ok()?)
          .cloned()
          .unwrap_or_default(),
      ),
      _ => None,
    }
  }

  pub fn set_instance_property(&self, key: &str, value: Value, is_public: bool) -> Option<Value> {
    match self {
      Self::Map(_, instance) => instance.on_ok(|v| v.set_instance_property(key, value, is_public)),
      Self::Class(class) => class.borrow().set_instance_property(key, value),
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
      (Self::Class(class), key) => class.borrow().get_instance_property(key),
      (Self::Array(array), "longitud") => Some(Value::Number(array.borrow().len().into())),
      (value, key) => {
        let proto_cache = thread
          .get_async()
          .borrow()
          .get_module()
          .borrow()
          .get_vm()
          .borrow()
          .cache
          .proto
          .clone();
        crate::interpreter::proto::proto(value.get_type().to_string(), proto_cache.clone())?
          .get_instance_property(key, thread)
      }
    }
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
      Self::Function(f) => f.borrow().to_string(),
      Self::Map(_, i) => i
        .on_some(|t| format!("<Instancia {}>", t.get_type()))
        .unwrap_or("<objeto>".to_string()),
      Self::Class(c) => {
        let has_parent = c.borrow().has_parent();
        let class = c.borrow().get_type().to_string();
        format!(
          "<clase {}>",
          if has_parent {
            let c_ref = c.borrow();
            let parent = c_ref.get_parent();

            let parent_ref = parent.borrow();
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
      Object::Function(function) => function.borrow().encode(),
      Object::Array(_) => Ok(vec![StructTag::Array as u8]),
      Object::Class(c) => {
        let mut encode = vec![StructTag::Class as u8];
        encode.extend(c.borrow().get_type().to_string().encode()?);
        Ok(encode)
      }
    }
  }
}
