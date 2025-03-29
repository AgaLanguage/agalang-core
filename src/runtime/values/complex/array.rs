use std::sync::{Arc, RwLock};

use crate::{
  functions_names, libraries, parser,
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
  util::OnError as _,
};

use super::AgalComplex;

#[derive(Clone, Debug)]
pub struct AgalArray(Arc<RwLock<Vec<values::DefaultRefAgalValue>>>);

impl AgalArray {
  pub fn new(vec: Vec<values::DefaultRefAgalValue>) -> Self {
    Self(Arc::new(RwLock::new(vec)))
  }
  pub fn to_vec(&self) -> Arc<RwLock<Vec<values::DefaultRefAgalValue>>> {
    self.0.clone()
  }
  pub fn to_buffer(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<Vec<u8>, internal::AgalThrow> {
    let mut buffer = vec![];
    let vec = &*self.0.as_ref().read().unwrap();
    for value in vec {
      let byte = value.to_agal_byte(stack.clone(), modules.clone());
      if let Err(value) = byte {
        return Err(value);
      }
      buffer.push(byte?.to_u8());
    }
    Ok(buffer)
  }
  pub fn set(
    &self,
    index: usize,
    value: values::DefaultRefAgalValue,
  ) -> values::DefaultRefAgalValue {
    let mut borrowed = &mut *self.0.write().unwrap();
    if index >= borrowed.len() {
      borrowed.extend(
        std::iter::repeat(values::AgalValue::Never.to_ref_value())
          .take((index + 1) - borrowed.clone().len()),
      );
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
impl From<&Vec<u8>> for AgalArray {
  fn from(buffer: &Vec<u8>) -> Self {
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
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    let mut result = String::new();
    let vec = self.to_vec();
    let vec = &*vec.read().unwrap();
    for (i, value) in vec.iter().enumerate() {
      let str = value.try_to_string(stack.clone(), modules.clone())?;
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
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    let mut result = String::new();
    let vec = self.to_vec();
    let vec = &*vec.read().unwrap();
    result.push_str("[");
    for (i, value) in vec.iter().enumerate() {
      let str = value
        .to_agal_console(stack.clone(), modules.clone())?
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
    (0..self.0.read().unwrap().len()).map(|i| format!("{i}")).collect()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(if (self.0.read().unwrap().len() == 0) {
      primitive::AgalBoolean::False
    } else {
      primitive::AgalBoolean::True
    })
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<values::RefAgalValue<AgalArray>, internal::AgalThrow> {
    Ok(self.clone().as_ref())
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Complex(c) = right.un_ref() {
      c.un_ref()
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: format!(
          "No se puede operar '{} {} {}'",
          self.get_name(),
          operator,
          right.get_name()
        ),
        stack,
      }
      .to_result();
    };
    match (other.clone(), operator) {
      (AgalComplex::Array(a), parser::NodeOperator::Plus) => {
        let vec = self.to_vec();
        let other_vec = a.un_ref().to_vec();
        let result = vec![vec.read().unwrap().clone(), other_vec.read().unwrap().clone()].concat();
        AgalArray::new(result).to_result()
      }
      (AgalComplex::Array(a), parser::NodeOperator::GreaterThanOrEqual) => {
        let ref l = a.un_ref();
        AgalBoolean::new(l.less_than(self) || self.equals(l)).to_result()
      }
      (AgalComplex::Array(a), parser::NodeOperator::LessThanOrEqual) => {
        let ref l = a.un_ref();
        AgalBoolean::new(self.less_than(l) || self.equals(l)).to_result()
      }
      (AgalComplex::Array(a), parser::NodeOperator::GreaterThan) => {
        let ref l = a.un_ref();
        AgalBoolean::new(l.less_than(self)).to_result()
      }
      (AgalComplex::Array(a), parser::NodeOperator::LessThan) => {
        let ref l = a.un_ref();
        AgalBoolean::new(self.less_than(l)).to_result()
      }
      (_, parser::NodeOperator::Nullish) => self.clone().to_result(),
      (_, parser::NodeOperator::Or) => {
        if self.to_agal_boolean(stack, modules)?.as_bool() == true {
          self.clone().to_result()
        } else {
          other.to_result()
        }
      }
      (_, parser::NodeOperator::And) => {
        if self.to_agal_boolean(stack, modules)?.as_bool() == false {
          self.clone().to_result()
        } else {
          other.to_result()
        }
      }
      (_, _) => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: format!(
          "No se puede operar '{} {} {}'",
          self.get_name(),
          operator,
          right.get_name()
        ),
        stack,
      }
      .to_result(),
    }
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let is_number = usize::from_str_radix(key, 10).on_error(|_| AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_string(),
      stack: stack.clone(),
    })?;
    self
      .to_vec()
      .read().unwrap()
      .get(is_number)
      .on_error(|_| AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: format!("'{}' no es un indice valido", is_number),
        stack,
      })?
      .clone()
      .to_result()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let index = usize::from_str_radix(key, 10).on_error(|_| AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_string(),
      stack: stack.clone(),
    })?;
    let len = self.to_vec().read().unwrap().len();
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
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      functions_names::TO_AGAL_STRING => modules
        .get_module(":proto/Lista")
        .ok_or_else(|| internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
          stack: stack.clone(),
        })?
        .get_instance_property(stack, key, modules),
      "longitud" => {
        let length = self.to_vec().read().unwrap().len();
        AgalNumber::Integer(length as i32).to_result()
      }
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: format!(
          "No se puede acceder a la propiedad '{}' de {}",
          key,
          self.get_name()
        ),
        stack,
      }
      .to_result(),
    }
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    let len = self.to_vec().read().unwrap().len();
    Ok(primitive::AgalNumber::Integer(len as i32))
  }

  fn equals(&self, other: &Self) -> bool {
    let vec = self.to_vec();
    let vec = vec.read().unwrap();
    let other_vec = other.to_vec();
    let other_vec = other_vec.read().unwrap();
    if vec.len() != other_vec.len() {
      return false;
    }
    for (a, b) in vec.iter().zip(other_vec.iter()) {
      if !a.equals(b) {
        return false;
      }
    }
    true
  }

  fn less_than(&self, other: &Self) -> bool {
    let vec = self.to_vec();
    let other_vec = other.to_vec();
    let x = vec.read().unwrap().clone().len() < other_vec.read().unwrap().clone().len();
    x
  }
}

impl IntoIterator for AgalArray {
  type Item = values::DefaultRefAgalValue;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.read().unwrap().clone().into_iter()
  }
}
