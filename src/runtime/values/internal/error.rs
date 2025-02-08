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
  fn get_name(&self) -> String {
    "Error".to_string()
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, super::throw::AgalThrow> {
    let (type_error, message) = self.get_data();
    let message = error_to_string(&type_error, message);
    Ok(primitive::AgalString::from_string(message))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalByte, super::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, super::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, super::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalNumber, super::AgalThrow> {
    todo!()
  }
}
impl traits::ToAgalValue for AgalError {
  fn to_value(self) -> AgalValue {
    AgalInternal::Error(self).to_value()
  }
}
