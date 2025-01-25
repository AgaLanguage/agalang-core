use parser::{
  internal::{error_to_string, ErrorNames, ErrorTypes},
  util::RefValue,
};

use crate::runtime::{
  stack::Stack,
  values::{
    self, primitive,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
};

use super::AgalInternal;

#[derive(Clone)]
pub enum AgalError {
  Params {
    type_error: ErrorNames,
    message: String,
    stack: RefValue<Stack>,
  },
  Value(values::DefaultRefAgalValue),
}

impl AgalError {
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

impl traits::AgalValuable for AgalError {
  fn to_agal_string(&self) -> Result<primitive::AgalString, super::throw::AgalThrow> {
    let (type_error, message) = self.get_data();
    let message = error_to_string(&type_error, message);
    Ok(primitive::AgalString::from_string(message))
  }
}
impl traits::ToAgalValue for AgalError {
  fn to_value(self) -> AgalValue {
    AgalInternal::Error(self).to_value()
  }
}
