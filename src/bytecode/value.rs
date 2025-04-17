use crate::{
  parser::{NodeFunction, NodeIdentifier},
  util::List,
};
use std::collections::HashMap;

use super::chunk::ChunkGroup;

type Number = f64;

#[derive(Clone, PartialEq)]
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
}
impl Function {
  pub fn chunk(&mut self) -> &mut ChunkGroup {
    match self {
      Self::Function { chunk, .. } => chunk,
      Self::Script { chunk, .. } => chunk,
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
    }
  }
}
impl ToString for Function {
  fn to_string(&self) -> String {
    match self {
      Self::Function { name, .. } => format!("<fn {name}>"),
      Self::Script { path, .. } => format!("<script '{path}'>"),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Object(HashMap<String, Value>),
  String(String),
  Function(Function),
}
impl Object {
  pub fn isString(&self) -> bool {
    match self {
      Self::String(_) => true,
      _ => false,
    }
  }
  pub fn isObject(&self) -> bool {
    match self {
      Self::Object(_) => true,
      _ => false,
    }
  }
  pub fn isFunction(&self) -> bool {
    match self {
      Self::Function { .. } => true,
      _ => false,
    }
  }
  pub fn asString(&self) -> String {
    match self {
      Self::Object(_) => "[objeto Objeto]".to_string(),
      Self::String(s) => s.clone(),
      Self::Function(f) => f.to_string(),
    }
  }
  pub fn asObject(&self) -> HashMap<String, Value> {
    match self {
      Self::Object(x) => x.clone(),
      _ => HashMap::new(),
    }
  }
  pub fn asFunction(&self) -> Function {
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
}
impl From<&str> for Object {
  fn from(value: &str) -> Self {
    Self::String(value.to_string())
  }
}
impl From<Function> for Object {
  fn from(value: Function) -> Self {
    Self::Function(value)
  }
}

pub const NULL_NAME: &str = "nulo";
pub const NEVER_NAME: &str = "nada";
pub const TRUE_NAME: &str = "cierto";
pub const FALSE_NAME: &str = "falso";

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Number(Number),
  Object(Object),
  False,
  True,
  Null,
  Never,
}
impl Value {
  pub fn isNumber(&self) -> bool {
    match self {
      Self::Number(_) => true,
      _ => false,
    }
  }
  pub fn isBoolean(&self) -> bool {
    match self {
      Self::False | Self::True => true,
      _ => false,
    }
  }
  pub fn isNull(&self) -> bool {
    match self {
      Self::Null => true,
      _ => false,
    }
  }
  pub fn isObject(&self) -> bool {
    match self {
      Self::Object(_) => true,
      _ => false,
    }
  }
  pub fn asNumber(&self) -> Number {
    match self {
      Self::Number(x) => *x,
      Self::True => 1.0,
      Self::Null | Self::Never | Self::False => 0.0,
      Self::Object(_) => 1.0,
    }
  }
  pub fn asBoolean(&self) -> bool {
    match self {
      Self::Number(x) => x != &0.0,
      Self::True => true,
      Self::Null | Self::Never | Self::False => false,
      Self::Object(_) => true,
    }
  }
  pub fn asObject(&self) -> Object {
    match self {
      Self::Number(x) => format!("{x}").as_str().into(),
      Self::True => TRUE_NAME.into(),
      Self::Null => NULL_NAME.into(),
      Self::Never => NEVER_NAME.into(),
      Self::False => FALSE_NAME.into(),
      Self::Object(x) => x.clone(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueArray {
  values: Vec<Value>,
}
impl ValueArray {
  pub fn new() -> Self {
    Self { values: Vec::new() }
  }
  fn init(&mut self) {
    self.values = vec![];
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
  pub fn get(&self, index: u8) -> &Value {
    self.values.get(index as usize).unwrap()
  }
}
