use std::{cell::RefCell, rc::Rc};

use crate::{
  libraries, parser,
  runtime::{
    self,
    values::{
      self, error_message, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
    RefStack,
  },
};

use super::AgalInternal;

#[derive(Clone, Debug)]
pub enum AgalThrow {
  Params {
    type_error: parser::ErrorNames,
    message: String,
    stack: runtime::RefStack,
  },
  Value(values::DefaultRefAgalValue),
}
impl AgalThrow {
  pub fn to_result<T>(self) -> Result<T, Self> {
    Err(self)
  }
  pub fn to_error(&self) -> super::AgalError {
    match self {
      Self::Params {
        type_error,
        message,
        ..
      } => super::AgalError::Params {
        type_error: type_error.clone(),
        message: message.clone(),
      },
      Self::Value(value) => super::AgalError::Value(value.clone()),
    }
  }
  pub fn get_data(&self) -> (parser::ErrorNames, parser::ErrorTypes) {
    match self {
      Self::Params {
        type_error,
        message,
        stack,
      } => {
        let mut string = String::new();
        string.push_str(message);
        for frame in stack {
          if let parser::Node::Block(..) = frame.as_ref() {
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

        (type_error.clone(), parser::ErrorTypes::StringError(string))
      }
      Self::Value(value) => (
        parser::ErrorNames::None,
        parser::ErrorTypes::StringError(value.as_string()),
      ),
    }
  }
}
impl traits::AgalValuable for AgalThrow {
  fn get_name(&self) -> String {
    "Lanzado".to_string()
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, AgalThrow> {
    Ok(primitive::AgalString::from_string(self.to_string()))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, AgalThrow> {
    self.to_agal_string(stack, modules)
  }

  fn equals(&self, other: &Self) -> bool {
    false
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}

impl ToString for AgalThrow {
  fn to_string(&self) -> String {
    let (type_error, message) = self.get_data();
    parser::error_to_string(&type_error, message)
  }
}
