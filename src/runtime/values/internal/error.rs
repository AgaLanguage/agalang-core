use crate::{
  libraries, parser,
  runtime::{
    self,
    stack::{self, RefStack},
    values::{
      self, error_message, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
};

use super::AgalInternal;

#[derive(Clone, Debug)]
pub enum AgalError {
  Params {
    type_error: parser::ErrorNames,
    message: String,
  },
  Value(values::DefaultRefAgalValue),
}

impl AgalError {
  pub fn get_data(&self) -> (parser::ErrorNames, parser::ErrorTypes) {
    match self {
      Self::Params {
        type_error,
        message,
      } => (
        type_error.clone(),
        parser::ErrorTypes::StringError(message.clone()),
      ),
      Self::Value(value) => (
        parser::ErrorNames::None,
        parser::ErrorTypes::StringError(value.as_string()),
      ),
    }
  }
}

impl traits::AgalValuable for AgalError {
  fn get_name(&self) -> String {
    "Error".to_string()
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, super::throw::AgalThrow> {
    let (type_error, message) = self.get_data();
    let message = parser::error_to_string(&type_error, message);
    Ok(primitive::AgalString::from_string(message))
  }
  fn to_agal_console(
      &self,
      stack: runtime::RefStack,
      modules: libraries::RefModules,
    ) -> Result<primitive::AgalString, super::AgalThrow> {
      Ok(self.to_agal_string(stack, modules)?.set_color(crate::util::Color::RED))
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, super::AgalThrow> {
    Ok(primitive::AgalBoolean::True)
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    match key {
      "tipo" => primitive::AgalString::from_string(format!("{}", self.get_data().0)).to_result(),
      "mensaje" => primitive::AgalString::from_string(format!("{}", self.get_data().1)).to_result(),
      _ => super::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_INSTANCE_PROPERTY.into(),
        stack,
      }
      .to_result(),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    false
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
impl traits::ToAgalValue for AgalError {
  fn to_value(self) -> AgalValue {
    AgalInternal::Error(self).to_value()
  }
}
