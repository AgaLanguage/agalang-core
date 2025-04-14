use std::collections::HashMap;

type Number = f64;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Object(HashMap<String, Value>),
  String(String)
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
  pub fn asString(&self) -> String {
    match self {
      Self::Object(_) => "[objeto Objeto]".to_string(),
      Self::String(s)=>s.clone()
    }
  }
  pub fn asObject(&self) -> HashMap<String, Value> {
    match self {
      Self::Object(x) => x.clone(),
      Self::String(s)=> HashMap::new()
    }
  }
}
impl From<&str> for Object {
  fn from(value: &str) -> Self {
    Self::String(value.to_string())
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
  Never
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

#[derive(Debug, Clone)]
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
    self.values.push(value);
  }
  pub fn len(&self) -> usize {
    self.values.len()
  }
  pub fn get(&self, index: usize) -> &Value {
    self.values.get(index).unwrap()
  }
}
