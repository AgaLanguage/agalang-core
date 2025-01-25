use std::{cell::RefCell, rc::Rc};

use parser::{
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

#[derive(Clone)]
pub enum AgalThrow {
  Params {
    type_error: ErrorNames,
    message: String,
    stack: RefValue<runtime::Stack>,
  },
  Value(values::DefaultRefAgalValue),
}
impl AgalThrow {
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
  pub fn get_data(&self) -> (ErrorNames, ErrorTypes) {
    match self {
      Self::Params {
        type_error,
        message,
        ..
      } => (type_error.clone(), ErrorTypes::StringError(message.clone())),
      Self::Value(value) => {
        let message = value.try_to_string();
        match message {
          Ok(message) => (ErrorNames::None, ErrorTypes::StringError(message)),
          Err(throw) => throw.get_data(),
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
    Err(self)
  }
}
impl traits::AgalValuable for AgalThrow {
  fn to_agal_string(&self) -> Result<primitive::AgalString, AgalThrow> {
    let (type_error, message) = self.get_data();
    let message = error_to_string(&type_error, message);
    Ok(primitive::AgalString::from_string(message))
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, AgalThrow> {
    self.to_agal_string()
  }
}
