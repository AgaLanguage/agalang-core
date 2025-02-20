use std::{cell::RefCell, rc::Rc};

use parser::{
  ast::Node,
  internal::{error_to_string, ErrorNames, ErrorTypes},
  util::RefValue,
};

use crate::{
  runtime::{
    self,
    values::{
      self, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  Modules,
};

use super::AgalInternal;

#[derive(Clone, Debug)]
pub enum AgalThrow {
  Params {
    type_error: ErrorNames,
    message: String,
    stack: runtime::RefStack,
  },
  Value(values::DefaultRefAgalValue),
}
impl AgalThrow {
  pub fn throw<T>(self) -> Result<T, Self> {
    Err(self)
  }
  pub fn to_error(&self) -> super::AgalError {
    match self {
      Self::Params {
        type_error,
        message,
        stack,
      } => super::AgalError::Params {
        type_error: type_error.clone(),
        message: message.clone(),
        stack: stack.clone(),
      },
      Self::Value(value) => super::AgalError::Value(value.clone()),
    }
  }
  pub fn get_data(&self, stack: runtime::RefStack) -> (ErrorNames, ErrorTypes) {
    match self {
      Self::Params {
        type_error,
        message,
        stack,
      } => {
        let mut string = String::new();
        string.push_str(message);
        for frame in stack {
          if let Node::Block(_) = frame.as_ref() {
            continue;
          }
          let location = frame.get_location();
          string.push_str(&match frame.get_type() {
            "Programa" => format!("\n  en {}", location.file_name),
            str => format!(
              "\n  {str} en {}:{}:{}",
              location.file_name,
              location.start.line + 1,
              location.start.column + 1
            ),
          });
        }

        (type_error.clone(), ErrorTypes::StringError(string))
      }
      Self::Value(value) => {
        let message = value.try_to_string(stack.clone());
        match message {
          Ok(message) => (ErrorNames::None, ErrorTypes::StringError(message)),
          Err(throw) => throw.get_data(stack),
        }
      }
    }
  }
}
impl traits::ToAgalValue for AgalThrow {
  fn to_value(self) -> AgalValue {
    AgalInternal::Throw(self).to_value()
  }
  fn to_result(self) -> Result<values::DefaultRefAgalValue, AgalThrow>
  where
    Self: Sized,
  {
    self.throw()
  }
}
impl traits::AgalValuable for AgalThrow {
  fn get_name(&self) -> String {
    "Lanzado".to_string()
  }
  fn to_agal_string(&self, stack: runtime::RefStack) -> Result<primitive::AgalString, AgalThrow> {
    Ok(primitive::AgalString::from_string(self.to_string()))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalString, AgalThrow> {
    self.to_agal_string(stack)
  }

  fn get_keys(&self) -> Vec<String> {
    vec![]
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, super::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, super::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, super::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: runtime::RefStack,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  async fn call(
    &mut self,
    stack: runtime::RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, super::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    todo!()
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}

impl ToString for AgalThrow {
  fn to_string(&self) -> String {
    let (type_error, message) = self.get_data(Default::default());
    error_to_string(&type_error, message)
  }
}
