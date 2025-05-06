#![allow(dead_code)]
use std::{
  cell::RefCell, collections::{HashMap, HashSet}, hash::{Hash, Hasher}, rc::Rc
};

use crate::{bytecode::ChunkGroup, parser::NodeFunction, util::cache::DataManager};
mod number;
pub use number::Number;

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Function {
  Function {
    arity: usize,
    chunk: ChunkGroup,
    name: String,
    is_async: bool,
    file: String,
  },
  Script {
    chunk: ChunkGroup,
    path: String,
  },
  Native {
    name: String,
    path: String,
    chunk: ChunkGroup,
    func: fn(Value, Vec<Value>) -> Result<Value, String>,
  },
}
impl Function {
  pub const fn get_type(&self) -> &str {
    match self {
      Self::Function { .. } => "funcion",
      Self::Script { .. } => "script",
      Self::Native { .. } => "nativo",
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
        file,
        ..
      } => format!(
        "en {} <{file}>",
        if *is_async {
          format!("asinc {name}")
        } else {
          name.to_string()
        }
      ),
      Self::Script { path, .. } => format!("en <{path}>"),
      Self::Native { path, .. } => format!("en <{path}>"),
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
      file: value.file.clone(),
    }
  }
}
impl std::fmt::Debug for Function {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Object {
  Map(
    MultiRefHash<HashMap<String, Value>>,
    MultiRefHash<HashMap<String, Value>>,
  ),
  Set(MultiRefHash<HashSet<Value>>),
  Array(MultiRefHash<Vec<Value>>),
  Function(Function),
}
impl Object {
  pub fn new() -> Self {
    Self::Map(HashMap::new().into(), HashMap::new().into())
  }
  pub const fn get_type(&self) -> &str {
    match self {
      Self::Function(f) => f.get_type(),
      Self::Map(_, _) => "objeto",
      Self::Array(_) => "lista",
      Self::Set(_) => "conjunto",
    }
  }
  pub fn is_map(&self) -> bool {
    match self {
      Self::Map(_, _) => true,
      _ => false,
    }
  }
  pub fn is_function(&self) -> bool {
    match self {
      Self::Function { .. } => true,
      _ => false,
    }
  }
  pub fn is_array(&self) -> bool {
    match self {
      Self::Array(_) => true,
      _ => false,
    }
  }
  pub fn as_map(
    &self,
  ) -> (
    MultiRefHash<HashMap<String, Value>>,
    MultiRefHash<HashMap<String, Value>>,
  ) {
    match self {
      Self::Map(x, y) => (x.clone(), y.clone()),
      _ => (HashMap::new().into(), HashMap::new().into()),
    }
  }
  pub fn as_array(&self) -> MultiRefHash<Vec<Value>> {
    match self {
      Self::Array(x) => x.clone(),
      _ => vec![].into(),
    }
  }
  pub fn as_function(&self) -> Function {
    match self {
      Self::Function(f) => f.clone(),
      _ => Function::Function {
        arity: 0,
        chunk: ChunkGroup::new(),
        name: "[Funcion invalida]".into(),
        is_async: false,
        file: "<nativo>".into(),
      },
    }
  }
  pub fn get_property(&self, key: &str) -> Option<Value> {
    match self {
      Self::Map(map, _) => map.borrow().get(key).cloned(),
      Self::Array(array) => {
        if let Ok(index) = key.parse::<usize>() {
          array.borrow().get(index).cloned()
        } else {
          None
        }
      }
      _ => None,
    }
  }
  pub fn get_instance_property(&self, key: &str, proto_cache: DataManager<String, Value>) -> Option<Value> {
    match (self, key) {
      (Self::Map(_, instance), key) => instance.borrow().get(key).cloned(),
      (Self::Array(array), "longitud") => Some(Value::Number(array.borrow().len().into())),
      (value, key) => super::proto::proto(value.get_type().to_string(), proto_cache.clone()).get_instance_property(key, proto_cache),
    }
  }
}
impl From<Function> for Object {
  fn from(value: Function) -> Self {
    Self::Function(value)
  }
}
impl From<HashMap<String, Value>> for Object {
  fn from(value: HashMap<String, Value>) -> Self {
    Self::Map(value.into(), HashMap::new().into())
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
      Self::Function(f) => f.to_string(),
      Self::Map(_, _) => format!("<Objeto {}>", self.get_type()),
      _ => format!("<{}>", self.get_type()),
    }
  }
}

pub const NULL_NAME: &str = "nulo";
pub const NEVER_NAME: &str = "nada";
pub const TRUE_NAME: &str = "cierto";
pub const FALSE_NAME: &str = "falso";

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum Value {
  Number(Number),
  String(String),
  Object(Object),
  Char(char),
  Byte(u8),
  False,
  True,
  Null,
  #[default]
  Never,
}
impl Value {
  pub const fn get_type(&self) -> &str {
    match self {
      Self::False | Self::True => "buleano",
      Self::Never => "nada",
      Self::Null => "nulo",
      Self::Number(_) => "numero",
      Self::String(_) => "cadena",
      Self::Char(_) => "caracter",
      Self::Byte(_) => "byte",
      Self::Object(o) => o.get_type(),
    }
  }
  pub fn is_number(&self) -> bool {
    match self {
      Self::Number(_) => true,
      _ => false,
    }
  }
  pub fn is_boolean(&self) -> bool {
    match self {
      Self::False | Self::True => true,
      _ => false,
    }
  }
  pub fn is_null(&self) -> bool {
    match self {
      Self::Null => true,
      _ => false,
    }
  }
  pub fn is_object(&self) -> bool {
    match self {
      Self::Object(_) => true,
      _ => false,
    }
  }
  pub fn is_string(&self) -> bool {
    match self {
      Self::String(_) => true,
      _ => false,
    }
  }

  pub fn as_number(&self) -> Number {
    match self {
      Self::Number(x) => x.clone(),
      Self::True => 1.into(),
      Self::Null | Self::Never | Self::False => 0.into(),
      Self::String(s) => s.parse::<Number>().unwrap_or(0.into()),
      Self::Object(_) => 1.into(),
      Self::Byte(b) => b.to_string().parse::<Number>().unwrap_or(0.into()),
      Self::Char(c) => c.into(),
    }
  }
  pub fn as_boolean(&self) -> bool {
    match self {
      Self::Char(c) => *c != '\0',
      Self::Number(x) => !x.is_zero(),
      Self::String(s) => !s.is_empty(),
      Self::Object(_) | Self::True => true,
      Self::Byte(b) => *b != 0,
      Self::Null | Self::Never | Self::False => false,
    }
  }
  pub fn as_object(&self) -> Object {
    match self {
      Self::String(s) => s.as_str().into(),
      Self::Object(x) => x.clone(),
      _ => Object::new(),
    }
  }
  pub fn as_string(&self) -> String {
    match self {
      Self::String(s) => s.clone(),
      Self::False => FALSE_NAME.to_string(),
      Self::True => TRUE_NAME.to_string(),
      Self::Null => NULL_NAME.to_string(),
      Self::Never => NEVER_NAME.to_string(),
      Self::Byte(b) => format!("0by{b:02X}"),
      Self::Number(x) => x.to_string(),
      Self::Char(c) => c.to_string(),
      Self::Object(x) => x.to_string(),
    }
  }
  pub fn get_instance_property(&self, key: &str, proto_cache: DataManager<String, Value>) -> Option<Value> {
    match (self, key) {
      (Self::Object(o), key) => o.get_instance_property(key, proto_cache),
      (Self::String(s), "longitud") => Some(Self::Number(s.len().into())),
      (Self::String(c), "bytes") => Some(Self::Object(
        c.as_bytes()
          .iter()
          .map(|&b| Self::Byte(b))
          .collect::<Vec<Self>>()
          .into(),
      )),
      (value, key) => super::proto::proto(value.get_type().to_string(), proto_cache.clone()).get_instance_property(key, proto_cache),
    }
  }
}
impl From<&str> for Value {
  fn from(value: &str) -> Self {
    Self::String(value.to_string())
  }
}
impl From<Number> for Value {
  fn from(value: Number) -> Self {
    Self::Number(value)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueArray {
  values: Vec<Value>,
}
impl ValueArray {
  pub fn new() -> Self {
    Self { values: Vec::new() }
  }
  pub fn write(&mut self, value: Value) {
    let index = self.values.len();
    if index >= 0xFF {
      self.values[index] = value;
      return;
    }
    self.values.push(value);
  }
  pub fn len(&self) -> u8 {
    self.values.len() as u8
  }
  pub fn try_get(&self, index: u8) -> Option<&Value> {
    if index as usize >= self.values.len() {
      return None;
    }
    Some(&self.values[index as usize])
  }
  pub fn get(&self, index: u8) -> &Value {
    self.values.get(index as usize).expect(&format!(
      "Error: el índice {} está fuera de rango (0-{})",
      index,
      self.values.len() - 1
    ))
  }
  pub fn has_value(&self, value: &Value) -> bool {
    self.values.iter().any(|v| v == value)
  }
  pub fn get_index(&self, value: &Value) -> Option<u8> {
    self.values.iter().position(|v| v == value).map(|i| i as u8)
  }
  pub fn enumerate(&self) -> impl Iterator<Item = (u8, &Value)> {
    self.values.iter().enumerate().map(|(i, v)| (i as u8, v))
  }
}
