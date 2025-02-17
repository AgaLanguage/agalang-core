use std::{cell::RefCell, rc::Rc};

use crate::{
  runtime::{
    self,
    values::{
      self, error_message,
      internal::{self, AgalThrow},
      primitive::{self, AgalBoolean, AgalNumber},
      traits::{self, AgalValuable, ToAgalValue as _},
      AgalValue,
    },
  },
  OnError, ToResult,
};
use parser::util::RefValue;

use super::AgalComplex;

#[derive(Clone, Debug)]
pub struct AgalArray(RefValue<Vec<values::DefaultRefAgalValue>>);

impl AgalArray {
  fn new(vec: Vec<values::DefaultRefAgalValue>) -> Self {
    Self(Rc::new(RefCell::new(vec)))
  }
  pub fn to_vec(&self) -> RefValue<Vec<values::DefaultRefAgalValue>> {
    self.0.clone()
  }
  pub fn to_buffer(&self, stack: runtime::RefStack) -> Result<Vec<u8>, internal::AgalThrow> {
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
  pub fn set(&self, index: usize, value: values::DefaultRefAgalValue) -> values::DefaultRefAgalValue {
    let mut borrowed = &mut *self.0.borrow_mut();
    if index >= borrowed.len() {
      borrowed.extend(std::iter::repeat(values::AgalValue::Never.to_ref_value()).take(index - borrowed.clone().len()));
    }
    borrowed[index] = value;
    borrowed[index].clone()
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
  fn to_agal_string(&self,stack: runtime::RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    let mut result = String::new();
    let vec = self.to_vec();
    let vec = &*vec.borrow();
    for (i, value) in vec.iter().enumerate() {
      let str = value.try_to_string(stack.clone())?;
      result.push_str(&str);
      if i < vec.len() - 1 {
        result.push_str(", ");
      }
    }
    Ok(primitive::AgalString::from_string(result))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
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
    (0..self.0.borrow().len()).map(|i| format!("{i}")).collect()
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::TO_AGAL_BYTE.to_string(),
      stack,
    }
    .throw()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(if (self.0.borrow().len() == 0) {
      primitive::AgalBoolean::False
    } else {
      primitive::AgalBoolean::True
    })
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<values::RefAgalValue<AgalArray>, internal::AgalThrow> {
    Ok(self.clone().as_ref())
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Complex(c) = right.un_ref() {
      c.un_ref()
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!(
          "No se puede operar '{} {} {}'",
          self.get_name(),
          operator,
          right.get_name()
        ),
        stack,
      }
      .throw();
    };
    match (other.clone(), operator) {
      (AgalComplex::Array(a), "+") => {
        let vec = self.to_vec();
        let other_vec = a.un_ref().to_vec();
        let result = vec![vec.borrow().clone(), other_vec.borrow().clone()].concat();
        AgalArray::new(result).to_result()
      }
      (AgalComplex::Array(a), ">=") => {
        let ref l = a.un_ref();
        AgalBoolean::new(l.less_than(self) || self.equals(l)).to_result()
      }
      (AgalComplex::Array(a), "<=") => {
        let ref l = a.un_ref();
        AgalBoolean::new(self.less_than(l) || self.equals(l)).to_result()
      }
      (AgalComplex::Array(a), ">") => {
        let ref l = a.un_ref();
        AgalBoolean::new(l.less_than(self)).to_result()
      }
      (AgalComplex::Array(a), "<") => {
        let ref l = a.un_ref();
        AgalBoolean::new(self.less_than(l)).to_result()
      }
      (_, "??") => self.clone().to_result(),
      (_, "||") => {
        if self.to_agal_boolean(stack)?.as_bool() == true {
          self.clone().to_result()
        } else {
          other.to_result()
        }
      }
      (_, "&&") => {
        if self.to_agal_boolean(stack)?.as_bool() == false {
          self.clone().to_result()
        } else {
          other.to_result()
        }
      }
      (_, _) => internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!(
          "No se puede operar '{} {} {}'",
          self.get_name(),
          operator,
          right.get_name()
        ),
        stack,
      }
      .throw(),
    }
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    if operator == "?" {
      self.clone().to_result()
    } else {
      internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!("No se puede aplicar '{}{}'", self.get_name(), operator),
        stack,
      }
      .throw()
    }
  }

  fn unary_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    match operator {
      "?" => self.to_agal_boolean(stack)?.to_result(),
      "!" => self.to_agal_boolean(stack)?.not().to_result(),
      "+" => self.to_agal_number(stack)?.to_result(),
      _ => internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!("No se puede aplicar '{}{}'", operator, self.get_name()),
        stack,
      }
      .throw(),
    }
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let is_number = usize::from_str_radix(key, 10).on_error(AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_string(),
      stack: stack.clone(),
    })?;
    self
      .to_vec()
      .borrow()
      .get(is_number)
      .on_error(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!("'{}' no es un indice valido", is_number),
        stack,
      })?
      .clone()
      .to_result()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let index = usize::from_str_radix(key, 10).on_error(AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_string(),
      stack: stack.clone(),
    })?;
    let len = self.to_vec().borrow().len();
    let index = if index <= 0 {
      0
    } else if index < len {
      index
    } else {
      len
    };
    self.set(index, value).to_result()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    if key == "longitud" {
      let length = self.to_vec().borrow().len();
      AgalNumber::Integer(length as i32).to_result()
    }else {
      internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!("No se puede acceder a la propiedad '{}' de {}", key, self.get_name()),
        stack,
      }
      .throw()
    }
  }

  async fn call(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::CALL.to_string(),
      stack,
    }
    .throw()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    let len = self.to_vec().borrow().len();
    Ok(primitive::AgalNumber::Integer(len as i32))
  }

  fn equals(&self, other: &Self) -> bool {
    todo!()
  }

  fn less_than(&self, other: &Self) -> bool {
    let vec = self.to_vec();
    let other_vec = other.to_vec();
    let x = vec.borrow().clone().len() < other_vec.borrow().clone().len();
    x
  }
}

impl IntoIterator for AgalArray {
  type Item = values::DefaultRefAgalValue;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.borrow().clone().into_iter()
  }
}