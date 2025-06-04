use crate::{
  bytecode::{proto, stack::VarsManager, value::class::Instance, vm::Thread, ChunkGroup},
  parser::NodeFunction,
  util::{Color, SetColor},
};
use std::{
  cell::RefCell,
  collections::HashMap,
  hash::{Hash, Hasher},
  rc::Rc,
};

use super::{class::Class, Value};

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Function {
  Function {
    arity: usize,
    chunk: ChunkGroup,
    name: String,
    is_async: bool,
    in_class: bool,
    location: crate::util::Location,
    scope: MultiRefHash<Option<Rc<RefCell<VarsManager>>>>,
    has_rest: bool,
  },
  Script {
    chunk: ChunkGroup,
    path: String,
    scope: MultiRefHash<Option<Rc<RefCell<VarsManager>>>>,
  },
  Native {
    name: String,
    path: String,
    chunk: ChunkGroup,
    func: fn(Value, Vec<Value>, &mut Thread) -> Result<Value, String>,
  },
}
impl Function {
  pub fn set_rest(&mut self, rest: bool) {
    match self {
      Self::Function { has_rest, .. } => *has_rest = rest,
      _ => {}
    }
  }
  pub fn get_type(&self) -> &'static str {
    match self {
      Self::Function { .. } => "funcion",
      Self::Script { .. } => "script",
      Self::Native { .. } => "nativo",
    }
  }
  pub fn set_in_class(&mut self) {
    match self {
      Self::Function { in_class, .. } => *in_class = true,
      Self::Script { .. } | Self::Native { .. } => {}
    }
  }
  pub fn get_in_class(&self) -> bool {
    match self {
      Self::Function { in_class, .. } => *in_class,
      Self::Script { .. } | Self::Native { .. } => false,
    }
  }
  pub fn set_scope(&self, vars: Rc<RefCell<VarsManager>>) {
    match self {
      Self::Function { scope: v, .. } => *v.borrow_mut() = Some(vars),
      Self::Script { scope: v, .. } => *v.borrow_mut() = Some(vars),
      Self::Native { .. } => {}
    }
  }
  pub fn get_scope(&self) -> Option<Rc<RefCell<VarsManager>>> {
    match self {
      Self::Function { scope: vars, .. } => vars.borrow().clone(),
      Self::Script { scope: vars, .. } => vars.borrow().clone(),
      Self::Native { .. } => None,
    }
  }
  pub fn chunk(&mut self) -> &mut ChunkGroup {
    match self {
      Self::Function { chunk, .. } => chunk,
      Self::Script { chunk, .. } => chunk,
      Self::Native { chunk, .. } => chunk,
    }
  }
  pub fn location(&self) -> String {
    match self {
      Self::Function {
        name,
        is_async,
        location,
        ..
      } => format!(
        "en {} <{}:{}:{}>",
        if *is_async {
          format!("asinc {name}")
        } else {
          name.to_string()
        },
        location.file_name.set_color(Color::Cyan),
        (location.start.line + 1)
          .to_string()
          .set_color(Color::Yellow),
        (location.start.column + 1)
          .to_string()
          .set_color(Color::Yellow)
      ),
      Self::Script { path, .. } => {
        format!(
          "en <{}:{}>",
          path.set_color(Color::Cyan),
          "script".to_string().set_color(Color::Gray)
        )
      }
      Self::Native { path, name, .. } => {
        if path.is_empty() {
          return format!(
            "en {name} <{}>",
            "nativo".to_string().set_color(Color::Gray)
          );
        }
        format!(
          "en {name} <{}:{}>",
          path.set_color(Color::Cyan),
          "nativo".to_string().set_color(Color::Gray)
        )
      }
    }
  }
}
impl ToString for Function {
  fn to_string(&self) -> String {
    match self {
      Self::Function { name, is_async, .. } => {
        format!("<{} {name}>", if *is_async { "asinc fn" } else { "fn" })
      }
      Self::Script { path, .. } => format!("<script '{path}'>"),
      Self::Native { name, .. } => format!("<nativo fn {name}>"),
    }
  }
}
impl From<&NodeFunction> for Function {
  fn from(value: &NodeFunction) -> Self {
    Self::Function {
      arity: value.params.len(),
      chunk: ChunkGroup::new(),
      name: value.name.clone(),
      is_async: value.is_async,
      location: value.location.clone(),
      scope: None.into(),
      has_rest: false,
      in_class: false,
    }
  }
}
impl std::fmt::Debug for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Object {
  Map(
    MultiRefHash<HashMap<String, Value>>,
    Option<MultiRefHash<Instance>>,
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

  pub fn set_instance_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      Self::Map(_, Some(instance)) => instance.borrow_mut().set_instance_property(key, value),
      Self::Class(class) => class.borrow().set_instance_property(key, value),
      _ => None,
    }
  }
  pub fn get_instance_property(&self, key: &str, thread: &Thread) -> Option<Value> {
    match (self, key) {
      (Self::Map(_, instance), key) => {
        if let Some(instance) = instance {
          instance.borrow().get_instance_property(key, thread)
        } else {
          None
        }
      }
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
        proto::proto(value.get_type().to_string(), proto_cache.clone())
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
    Self::Map(value.into(), None)
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
      Self::Map(_, _) => format!("<Objeto {}>", self.get_type()),
      _ => format!("<{}>", self.get_type()),
    }
  }
}
impl std::fmt::Debug for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
