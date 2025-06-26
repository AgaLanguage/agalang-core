use std::collections::HashMap;

mod class;
mod function;
mod number;
mod object;
mod promise;
pub use class::{Class, Instance};
pub use function::*;
pub use number::Number;
pub use object::*;
pub use promise::{Promise, PromiseData, PROMISE_TYPE};

use crate::interpreter::{Thread, VarsManager};
use crate::util::{MutClone, OnError, Valuable};
use crate::MultiRefHash;
use crate::{compiler::ChunkGroup, Decode, StructTag};

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
pub const LAZY_TYPE: &str = "vago";

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct RefValue(MultiRefHash<Value>);
impl RefValue {
  pub fn borrow(&self) -> std::sync::RwLockReadGuard<Value> {
    self.0.read()
  }
}
impl From<Value> for RefValue {
  fn from(value: Value) -> Self {
    Self(MultiRefHash::from(value))
  }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LazyValue {
  value: MultiRefHash<Option<Value>>,
  once: MultiRefHash<Function>,
}
impl LazyValue {
  pub fn get(&self) -> std::sync::RwLockReadGuard<Option<Value>> {
    self.value.read()
  }
  pub fn set(&self, value: Value) {
    *self.value.write() = Some(value)
  }
  pub fn get_once(&self) -> MultiRefHash<Function> {
    self.once.clone()
  }
}
impl From<Function> for LazyValue {
  fn from(value: Function) -> Self {
    Self {
      value: Default::default(),
      once: value.into(),
    }
  }
}

impl crate::Encode for LazyValue {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut vec = vec![StructTag::Lazy as u8];
    vec.extend(self.once.cloned().encode()?);
    Ok(vec)
  }
}
impl Decode for LazyValue {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec.pop_front();
    Ok(Self {
      once: Function::decode(vec)?.into(),
      value: Default::default(),
    })
  }
}

#[derive(Clone, PartialEq, Eq, Default, Hash)]
pub enum Value {
  Number(Number),
  String(String),
  Object(Object),
  Lazy(LazyValue),
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
      Self::Lazy { .. } => LAZY_TYPE,
      Self::Object(o) => o.get_type(),
    }
  }
  pub fn set_scope(&self, vars: MultiRefHash<VarsManager>) {
    match self {
      Self::Object(Object::Function(f)) => f.read().set_scope(vars),
      Self::Lazy(LazyValue { once, .. }) => once.read().set_scope(vars),
      _ => panic!(
        "Error: no se puede establecer una variable local en un valor de tipo {}",
        self.get_type()
      ),
    }
  }
  pub fn set_in_class(&self, class: MultiRefHash<Class>) {
    match self {
      Self::Object(Object::Function(f)) => f.write().set_in_class(class),
      _ => {}
    }
  }
  pub fn is_number(&self) -> bool {
    match self {
      Self::Number(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_number(),
      _ => false,
    }
  }
  pub fn is_object(&self) -> bool {
    match self {
      Self::Object(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_object(),
      _ => false,
    }
  }
  pub fn is_string(&self) -> bool {
    match self {
      Self::String(_) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_string(),
      _ => false,
    }
  }
  pub fn is_iterator(&self) -> bool {
    match self {
      Self::Iterator(_) => true,
      Self::Ref(RefValue(r)) => r.read().is_iterator(),
      _ => false,
    }
  }
  pub fn is_promise(&self) -> bool {
    match self {
      Self::Promise(_) => true,
      Self::Ref(RefValue(r)) => r.read().is_promise(),
      _ => false,
    }
  }
  pub fn as_promise(&self) -> Promise {
    match self {
      Self::Promise(promise) => promise.clone(),
      Self::Ref(RefValue(r)) => r.read().as_promise(),
      Self::Lazy(lazy) => lazy.get().clone().unwrap_or_default().as_promise(),
      v => v.clone().into(),
    }
  }

  pub fn as_number(&self) -> Result<Number, String> {
    match self {
      Self::Number(x) => x.clone_ok(),
      val => Err(format!(
        "Se esperaba un '{NUMBER_TYPE}' pero se recibio un {}",
        val.get_type()
      )),
    }
  }
  pub fn as_boolean(&self) -> Result<bool, String> {
    match self {
      Self::True => Ok(true),
      Self::Null | Self::Never | Self::False => Ok(false),
      Self::Lazy(l) => l.get().clone().unwrap_or_default().as_boolean(),
      val => Err(format!(
        "Se esperaba un '{BOOLEAN_TYPE}' pero se recibio un {}",
        val.get_type()
      )),
    }
  }

  pub fn is_function(&self) -> bool {
    match self {
      Self::Object(Object::Function { .. }) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_function(),
      _ => false,
    }
  }
  pub fn is_class(&self) -> bool {
    match self {
      Self::Object(Object::Class { .. }) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_class(),
      _ => false,
    }
  }
  pub fn is_array(&self) -> bool {
    match self {
      Self::Object(Object::Array(_)) => true,
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().is_array(),
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
      Self::Iterator(v) => format!("@{}", v.read().as_string()),
      Self::Ref(v) => format!("&{}", v.borrow().as_string()),
      Self::Byte(b) => format!("0x{b:02X}"),
      Self::Number(x) => x.to_string(),
      Self::Char(c) => c.to_string(),
      Self::Object(x) => x.to_string(),
      Self::Lazy(l) => l.get().clone().unwrap_or_default().as_string(),
    }
  }
  pub fn as_function(&self) -> MultiRefHash<Function> {
    match self {
      Self::Object(Object::Function(f)) => f.clone(),
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().as_function(),
      _ => Function::Script {
        chunk: ChunkGroup::new().into(),
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
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().as_map(),
      _ => (HashMap::new().into(), None.into()),
    }
  }
  pub fn as_class(&self) -> MultiRefHash<Class> {
    match self {
      Self::Object(Object::Class(c)) => c.clone(),
      Self::Ref(RefValue(r)) | Self::Iterator(r) => r.read().as_class(),
      _ => Class::new("<nulo>".to_string()),
    }
  }
  pub fn as_strict_array(&self, thread: &Thread) -> Result<Vec<Value>, String> {
    match self {
      Self::Object(Object::Array(array)) => Ok(array.read().clone()),
      Self::Object(Object::Map(_, instance)) => {
        let instance = instance.cloned();
        if matches!(instance, None) {
          return Err(format!(
            "No se puede convertir a lista: {}",
            self.get_type()
          ));
        }
        let value = instance
          .unwrap()
          .get_instance_property(crate::functions_names::ARRAY, thread);
        if matches!(value, None) {
          return Err(format!(
            "No se puede convertir a lista: {}",
            self.get_type()
          ));
        }
        if let Value::Object(Object::Array(array)) = value.unwrap() {
          return array.read().clone_ok();
        }

        Err(format!(
          "No se puede convertir a lista: {}",
          self.get_type()
        ))
      }
      Self::Ref(RefValue(l)) | Self::Iterator(l) => {
        l.read().as_strict_array(thread).or_else(|_| {
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
  pub fn as_strict_byte(&self) -> Result<u8, String> {
    match self {
      Self::Ref(RefValue(l)) => l.read().as_strict_byte().or_else(|_| {
        Err(format!(
          "No se puede convertir a lista: {}",
          self.get_type()
        ))
      }),
      Self::Byte(b) => Ok(*b),
      _ => Err(format!("No se puede convertir a byte: {}", self.get_type())),
    }
  }
  pub fn as_strict_buffer(&self, thread: &Thread) -> Result<Vec<u8>, String> {
    match self {
      Self::Byte(b) => Ok(vec![*b]),
      Self::String(string) => Ok(string.as_bytes().to_vec()),
      Self::Ref(RefValue(l)) => l.read().as_strict_buffer(thread).or_else(|_| {
        Err(format!(
          "No se puede convertir a buffer: {}",
          self.get_type()
        ))
      }),
      value => value
        .as_strict_array(thread)
        .map_err(|e| {
          format!(
            "No se puede convertir a buffer ({}): {}",
            self.get_type(),
            e
          )
        })?
        .iter()
        .map(|v| v.as_strict_byte())
        .collect(),
    }
  }
  pub fn set_object_property(&self, key: &str, value: Value) -> Option<Value> {
    match self {
      Self::Object(o) => o.set_object_property(key, value),
      Self::Iterator(r) => r.read().set_object_property(key, value),
      _ => None,
    }
  }
  pub fn get_object_property(&self, key: &str) -> Option<Value> {
    match self {
      Self::Object(o) => o.get_object_property(key),
      Self::Ref(RefValue(r)) => r.read().get_object_property(key),
      Self::Iterator(r) => r.read().get_object_property(key),
      _ => None,
    }
  }
  pub fn set_instance_property(
    &self,
    key: &str,
    value: Value,
    is_public: bool,
    is_class_decl: bool,
    thread: &Thread,
  ) -> Option<Value> {
    match self {
      Self::Iterator(r) => {
        r.read()
          .set_instance_property(key, value, is_public, is_class_decl, thread)
      }
      Self::Object(Object::Map(_, instance)) => {
        let instance = instance.read();
        let instance = instance.as_ref()?;
        let class = thread.get_calls().last().unwrap().in_class();
        let assign = if let Some(class) = class {
          instance.is_instance(class.read().get_instance().read().clone().unwrap())
        } else {
          true
        };
        if assign {
          Some(instance.set_instance_property(key, value, is_public))
        } else {
          None
        }
      }
      Self::Object(Object::Class(class)) => {
        if is_class_decl {
          Some(class.read().set_instance_property(key, value))
        } else {
          None
        }
      }
      _ => None,
    }
  }
  pub fn get_instance_property(&self, key: &str, thread: &Thread) -> Option<Value> {
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
    match (self, key) {
      (Self::Ref(RefValue(r)), key) => r.read().get_instance_property(key, thread),
      (Self::Object(o), key) => o.get_instance_property(key, thread),
      (Self::String(s), "longitud") => Some(Self::Number(s.len().into())),
      (Self::String(c), "bytes") => Some(Self::Object(
        c.as_bytes()
          .iter()
          .map(|&b| Self::Byte(b))
          .collect::<Vec<Self>>()
          .into(),
      )),
      (value, key) => {
        crate::interpreter::proto::proto(value.get_type().to_string(), proto_cache.clone())?
          .get_instance_property(key, thread)
      }
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
    if let Self::String(string) = self {
      write!(f, "'{}'", string.replace("\n", "\\n").replace("\'", "\\\'"))
    } else {
      write!(f, "{}", self.as_string())
    }
  }
}
impl crate::Encode for Value {
  fn encode(&self) -> Result<Vec<u8>, String> {
    match self {
      Self::Byte(byte) => Ok(vec![StructTag::Byte as u8, *byte]),
      Self::False => false.encode(),
      Self::True => true.encode(),
      Self::Number(number) => number.encode(),
      Self::String(string) => string.encode(),
      Self::Object(object) => object.encode(),
      Self::Iterator(_) => Err("No se pueden compilar iteradores".to_string()),
      Self::Promise(_) => Err("No se pueden compilar promesas".to_string()),
      Self::Ref(_) => Err("No se pueden compilar referencias".to_string()),
      Self::Char(char) => char.encode(),
      Self::Null => Ok(vec![StructTag::Null as u8]),
      Self::Never => Ok(vec![StructTag::Never as u8]),
      Self::Lazy(l) => l.encode(),
    }
  }
}
impl Decode for Value {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    let tag_byte = (*vec.get(0).on_error(|_| "Se esperaba un valor")?).into();
    match tag_byte {
      StructTag::Byte => {
        vec.pop_front();
        Ok(Self::Byte(
          vec.pop_front().on_error(|_| "Binario corrupto")?,
        ))
      }
      StructTag::Bool => {
        vec.pop_front();
        Ok(if vec.pop_front().on_error(|_| "Binario corrupto")? == 0 {
          Self::False
        } else {
          Self::True
        })
      }
      StructTag::Number => Ok(Self::Number(Number::decode(vec)?)),
      StructTag::String => Ok(Self::String(String::decode(vec)?)),
      StructTag::Map => {
        vec.pop_front();
        Ok(Self::Object(Object::Map(
          Default::default(),
          Default::default(),
        )))
      }
      StructTag::Class => {
        vec.pop_front();
        Ok(Self::Object(Object::Class(Class::new(String::decode(
          vec,
        )?))))
      }
      StructTag::Function => Ok(Self::Object(Object::Function(
        Function::decode(vec)?.into(),
      ))),
      StructTag::Char => Ok(Self::Char(char::decode(vec)?)),
      StructTag::Null => {
        vec.pop_front();
        Ok(Self::Null)
      }
      StructTag::Never => {
        vec.pop_front();
        Ok(Self::Never)
      }
      StructTag::Lazy => Ok(Self::Lazy(LazyValue::decode(vec)?)),
      _ => Err("Se esperaba un valor".to_string()),
    }
  }
}
impl MutClone for Value {}

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
  pub fn enumerate(&self) -> impl Iterator<Item = (u8, &Value)> {
    self.values.iter().enumerate().map(|(i, v)| (i as u8, v))
  }
}
