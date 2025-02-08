use std::{cell::RefCell, rc::Rc};

use crate::runtime::{
  self,
  values::{
    self, internal, primitive,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
};
use parser::util::RefValue;

use super::AgalComplex;

#[derive(Clone)]
pub struct AgalArray(RefValue<Vec<values::DefaultRefAgalValue>>);

impl AgalArray {
  fn new(vec: Vec<values::DefaultRefAgalValue>) -> Self {
    Self(Rc::new(RefCell::new(vec)))
  }
  pub fn to_vec(&self) -> RefValue<Vec<values::DefaultRefAgalValue>> {
    self.0.clone()
  }
  pub fn to_buffer(&self, stack: RefValue<runtime::Stack>) -> Result<Vec<u8>, internal::AgalThrow> {
    let mut buffer = vec![];
    let vec = &*self.0.as_ref().borrow();
    for value in vec {
      let byte = value.to_agal_byte(stack.clone());
      if let Err(value) = byte {
        return Err(value);
      }
      buffer.push(byte?.to_u8());
    }
    Ok(buffer)
  }
}

impl From<&primitive::AgalString> for AgalArray {
  fn from(string: &primitive::AgalString) -> Self {
    let vec = string
      .to_agal_chars()
      .iter()
      .map(|c| c.to_ref_value())
      .collect();
    Self::new(vec)
  }
}

impl From<Vec<values::DefaultRefAgalValue>> for AgalArray {
  fn from(vec: Vec<values::DefaultRefAgalValue>) -> Self {
    Self::new(vec)
  }
}

impl From<&[u8]> for AgalArray {
  fn from(buffer: &[u8]) -> Self {
    let mut vec = Vec::new();
    for byte in buffer {
      vec.push(primitive::AgalByte::new(*byte).to_ref_value());
    }
    Self::new(vec)
  }
}

impl traits::ToAgalValue for AgalArray {
  fn to_value(self) -> AgalValue {
    AgalComplex::Array(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalArray {
  fn get_name(&self) -> String {
    "Lista".to_string()
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    let mut result = String::new();
    let vec = self.to_vec();
    let vec = &*vec.borrow();
    for (i, value) in vec.iter().enumerate() {
      let str = value.try_to_string()?;
      result.push_str(&str);
      if i < vec.len() - 1 {
        result.push_str(", ");
      }
    }
    Ok(primitive::AgalString::from_string(result))
  }
  fn to_agal_console(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    let mut result = String::new();
    let vec = self.to_vec();
    let vec = &*vec.borrow();
    result.push_str("[");
    for (i, value) in vec.iter().enumerate() {
      let str = value
        .to_agal_console(stack.clone(), env.clone())?
        .add_prev(" ")
        .to_string();
      result.push_str(&str);
      if i < vec.len() - 1 {
        result.push_str(",");
      }
    }
    result.push_str(" ]");
    Ok(primitive::AgalString::from_string(result))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<values::RefAgalValue<AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
}
