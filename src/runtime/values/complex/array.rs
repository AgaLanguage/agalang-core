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
  pub fn from_buffer(buffer: &[u8]) -> Self {
    let mut vec = Vec::new();
    for byte in buffer {
      vec.push(primitive::AgalByte::new(*byte).to_ref_value());
    }
    Self::new(vec)
  }
  pub fn from_vec(vec: Vec<values::DefaultRefAgalValue>) -> Self {
    Self::new(vec)
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
impl traits::ToAgalValue for AgalArray {
  fn to_value(self) -> AgalValue {
    AgalComplex::Array(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalArray {
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
}
