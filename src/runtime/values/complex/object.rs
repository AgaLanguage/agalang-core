use std::{
  cell::{Ref, RefCell},
  collections::HashMap,
  rc::Rc,
};

use crate::runtime::{
  self,
  values::{
    self, internal, primitive,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
};

use super::AgalComplex;

type AgalHashMap = HashMap<String, values::DefaultRefAgalValue>;
type RefAgalHashMap = Rc<RefCell<AgalHashMap>>;
type RefAgalProto = values::RefAgalValue<super::AgalPrototype>;
#[derive(Clone)]
pub struct AgalObject(RefAgalHashMap, Option<RefAgalProto>);

impl AgalObject {
  pub fn from_hashmap(hashmap: RefAgalHashMap) -> Self {
    Self(hashmap, None)
  }
  pub fn from_hashmap_with_prototype(hashmap: RefAgalHashMap, prototype: RefAgalProto) -> Self {
    Self(hashmap, Some(prototype))
  }
  pub fn from_prototype(hashmap: RefAgalProto) -> AgalObject {
    AgalObject(Rc::new(RefCell::new(HashMap::new())), Some(hashmap))
  }
  pub fn get_hashmap(&self) -> Ref<AgalHashMap> {
    self.0.as_ref().borrow()
  }
  pub fn get_prototype(&self) -> Option<RefAgalProto> {
    if let Some(a) = &self.1 {
      Some(a.clone())
    } else {
      None
    }
  }
}
impl traits::ToAgalValue for AgalObject {
  fn to_value(self) -> AgalValue {
    AgalComplex::Object(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalObject {
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string("<Objeto>".to_string()))
  }
  fn get_object_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> values::ResultAgalValue {
    let hashmap = &mut *self.0.as_ref().borrow_mut();
    match hashmap.get(key) {
      Some(v) => Ok(v.clone()),
      None => internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::LexerError,
        message: "No tengo esa propiedad de instancia".to_string(),
        stack,
      }
      .to_result(),
    }
  }
  fn set_object_property(
    &mut self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> values::ResultAgalValue {
    let hashmap = &mut *self.0.as_ref().borrow_mut();
    hashmap.insert(key.to_string(), value.clone());
    Ok(value.clone())
  }
  fn get_instance_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    if let Some(v) = {
      if let Some(v) = self.get_prototype() {
        v.borrow().get(key)
      } else {
        None
      }
    } {
      return Ok(v.value);
    }
    internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::LexerError,
      message: "No tengo esa propiedad de instancia".to_string(),
      stack,
    }
    .to_result()
  }
}
