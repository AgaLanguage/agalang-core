use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod class;
mod function;
mod number;
mod object;
mod promise;
pub use class::{Class, Instance};
pub use function::Function;
pub use number::Number;
pub use object::{MultiRefHash, Object};
pub use promise::{Promise, PromiseData, PROMISE_TYPE};

use crate::{compiler::ChunkGroup, interpreter::{Thread, VarsManager}};

pub const NULL_NAME: &str = "nulo";
pub const NEVER_NAME: &str = "nada";
pub const TRUE_NAME: &str = "cierto";
pub const FALSE_NAME: &str = "falso";

pub const STRING_TYPE: &str = "cadena";
pub const NUMBER_TYPE: &str = "numero";
pub const BOOLEAN_TYPE: &str = "buleano";
pub const ITERATOR_TYPE: &str = "iterador";
pub const REF_TYPE: &str = "referencia";
pub const CHAR_TYPE: &str = "caracter";
pub const BYTE_TYPE: &str = "byte";

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct RefValue(MultiRefHash<Value>);
impl RefValue {
  pub fn borrow(&self) -> std::cell::Ref<Value> {
    self.0.borrow()
  }
}
impl From<Value> for RefValue {
  fn from(value: Value) -> Self {
    Self(MultiRefHash::from(value))
  }
}

#[derive(Clone, PartialEq, Eq, Default, Hash)]
pub enum Value {
  Number(Number),
  String(String),
  Object(Object),
  Iterator(MultiRefHash<Value>),
  Promise(Promise),
  Ref(RefValue),
  Char(char),
  Byte(u8),
  False,
  True,
  Null,
  #[default]
  Never,
}
impl Value {
  pub fn get_type(&self) -> &str {
    match self {
      Self::False | Self::True => BOOLEAN_TYPE,
      Self::Never => NEVER_NAME,
      Self::Null => NULL_NAME,
      Self::Number(_) => NUMBER_TYPE,
      Self::String(_) => STRING_TYPE,
      Self::Char(_) => CHAR_TYPE,
      Self::Byte(_) => BYTE_TYPE,
      Self::Iterator(_) => ITERATOR_TYPE,
      Self::Ref(_) => REF_TYPE,
      Self::Promise(_) => PROMISE_TYPE,
      Self::Object(o) => o.get_type(),
    }
  }
  pub fn set_scope(&self, vars: Rc<RefCell<VarsManager>>) {
    match self {
      Self::Object(Object::Function(f)) => f.borrow().set_scope(vars),
      _ => panic!(
        "Error: no se puede establecer una variable local en un valor de tipo {}",
        self.get_type()
      ),
    }
  }
  pub fn set_in_class(&self, class: MultiRefHash<Class>) {
    match self {
      Self::Object(Object::Function(f)) => f.borrow_mut().set_in_class(class),
      _ => {}
    }
  }
  pub fn is_number(&self) -> bool {
    match self {
      Self::Number(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_number(),
      _ => false,
    }
  }
  pub fn is_object(&self) -> bool {
    match self {
      Self::Object(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_object(),
      _ => false,
    }
  }
  pub fn is_string(&self) -> bool {
    match self {
      Self::String(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_string(),
      _ => false,
    }
  }
  pub fn is_iterator(&self) -> bool {
    match self {
      Self::Iterator(_) => true,
      Self::Ref(RefValue(r)) => r.borrow().is_iterator(),
      _ => false,
    }
  }
  pub fn is_promise(&self) -> bool {
    match self {
      Self::Promise(_) => true,
      Self::Ref(RefValue(r)) => r.borrow().is_promise(),
      _ => false,
    }
  }
  pub fn as_promise(&self) -> Promise {
    match self {
      Self::Promise(promise) => promise.clone(),
      Self::Ref(RefValue(r)) => r.borrow().as_promise(),
      v => v.clone().into(),
    }
  }

  pub fn as_number(&self) -> Number {
    match self {
      Self::Promise(_) => 0.into(),
      Self::Number(x) => x.clone(),
      Self::True => 1.into(),
      Self::Null | Self::Never | Self::False => 0.into(),
      Self::String(s) => s.parse::<Number>().unwrap_or(0.into()),
      Self::Iterator(v) | Self::Ref(RefValue(v)) => v.borrow().as_number(),
      Self::Object(_) => 1.into(),
      Self::Byte(b) => b.to_string().parse::<Number>().unwrap_or(0.into()),
      Self::Char(c) => c.into(),
    }
  }
  pub fn as_boolean(&self) -> bool {
    match self {
      Self::Char(c) => *c != '\0',
      Self::Promise(x) => !matches!(x.get_data(), PromiseData::Pending),
      Self::Number(x) => !x.is_zero(),
      Self::String(s) => !s.is_empty(),
      Self::Iterator(v) | Self::Ref(RefValue(v)) => v.borrow().as_boolean(),
      Self::Object(_) | Self::True => true,
      Self::Byte(b) => *b != 0,
      Self::Null | Self::Never | Self::False => false,
    }
  }

  pub fn is_function(&self) -> bool {
    match self {
      Self::Object(Object::Function { .. }) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_function(),
      _ => false,
    }
  }
  pub fn is_class(&self) -> bool {
    match self {
      Self::Object(Object::Class { .. }) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_class(),
      _ => false,
    }
  }
  pub fn is_array(&self) -> bool {
    match self {
      Self::Object(Object::Array(_)) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().is_array(),
      _ => false,
    }
  }

  pub fn as_string(&self) -> String {
    match self {
      Self::String(s) => s.clone(),
      Self::Promise(p) => p.to_string(),
      Self::False => FALSE_NAME.to_string(),
      Self::True => TRUE_NAME.to_string(),
      Self::Null => NULL_NAME.to_string(),
      Self::Never => NEVER_NAME.to_string(),
      Self::Iterator(v) => format!("@{}", v.borrow().as_string()),
      Self::Ref(v) => format!("&{}", v.borrow().as_string()),
      Self::Byte(b) => format!("0x{b:02X}"),
      Self::Number(x) => x.to_string(),
      Self::Char(c) => c.to_string(),
      Self::Object(x) => x.to_string(),
    }
  }
  pub fn as_function(&self) -> MultiRefHash<Function> {
    match self {
      Self::Object(Object::Function(f)) => f.clone(),
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().as_function(),
      _ => Function::Script {
        chunk: ChunkGroup::new(),
        scope: None.into(),
        path: "<nulo>".to_string(),
      }
      .into(),
    }
  }
  pub fn as_map(
    &self,
  ) -> (
    MultiRefHash<HashMap<String, Value>>,
    MultiRefHash<Option<Instance>>,
  ) {
    match self {
      Self::Object(Object::Map(prop, instance)) => (prop.clone(), instance.clone()),
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().as_map(),
      _ => (HashMap::new().into(), None.into()),
    }
  }
  pub fn as_class(&self) -> MultiRefHash<Class> {
    match self {
      Self::Object(Object::Class(c)) => c.clone(),
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.borrow().as_class(),
      _ => Class::new("<nulo>".to_string()).0,
    }
  }
  pub fn as_strict_array(&self, thread: &Thread) -> Result<Vec<Value>, String> {
    match self {
      Self::Object(Object::Array(array)) => Ok(array.borrow().clone()),
      Self::Object(Object::Map(_, instance)) => {
        let instance = instance.cloned();
        if matches!(instance, None) {
          return Err(format!(
            "No se puede convertir a lista: {}",
            self.get_type()
          ));
        }
        let value = instance.unwrap().get_instance_property(crate::functions_names::TO_AGAL_ARRAY, thread);
        if matches!(value, None) {
          return Err(format!(
            "No se puede convertir a lista: {}",
            self.get_type()
          ));
        }
        if let Value::Object(Object::Array(array)) = value.unwrap() {
          return Ok(array.borrow().clone());
        }

        Err(format!(
          "No se puede convertir a lista: {}",
          self.get_type()
        ))
      }
      Self::Ref(RefValue(l)) | Self::Iterator(l) => {
        l.borrow().as_strict_array(thread).or_else(|_| {
          Err(format!(
            "No se puede convertir a lista: {}",
            self.get_type()
          ))
        })
      }
      Self::String(string) => {
        let chars = string.chars().map(Value::from).collect::<Vec<Value>>();
        Ok(chars)
      }
      _ => Err(format!(
        "No se puede convertir a lista: {}",
        self.get_type()
      )),
    }
  }
  pub fn set_object_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      Self::Object(o) => o.set_object_property(key, value),
      Self::Iterator(r) => r.borrow().set_object_property(key, value),
      _ => None,
    }
  }
  pub fn get_object_property(&self, key: &str) -> Option<Value> {
    match self {
      Self::Object(o) => o.get_object_property(key),
      Self::Ref(RefValue(r)) => r.borrow().get_object_property(key),
      Self::Iterator(r) => r.borrow().get_object_property(key),
      _ => None,
    }
  }
  pub fn set_instance_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      Self::Iterator(r) => r.borrow().set_instance_property(key, value),
      Self::Object(o) => o.set_instance_property(key, value),
      _ => None,
    }
  }
  pub fn get_instance_property(&self, key: &str, thread: &Thread) -> Option<Value> {
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
    match (self, key) {
      (Self::Ref(RefValue(r)), key) => r.borrow().get_instance_property(key, thread),
      (Self::Object(o), key) => o.get_instance_property(key, thread),
      (Self::String(s), "longitud") => Some(Self::Number(s.len().into())),
      (Self::String(c), "bytes") => Some(Self::Object(
        c.as_bytes()
          .iter()
          .map(|&b| Self::Byte(b))
          .collect::<Vec<Self>>()
          .into(),
      )),
      (value, key) => crate::interpreter::proto::proto(value.get_type().to_string(), proto_cache.clone())?
        .get_instance_property(key, thread),
    }
  }
}
impl From<char> for Value {
  fn from(value: char) -> Self {
    Self::Char(value)
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
impl From<bool> for Value {
  fn from(value: bool) -> Self {
    if value {
      Self::True
    } else {
      Self::False
    }
  }
}
impl std::fmt::Debug for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_string())
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
  pub fn _enumerate(&self) -> impl Iterator<Item = (u8, &Value)> {
    self.values.iter().enumerate().map(|(i, v)| (i as u8, v))
  }
}
