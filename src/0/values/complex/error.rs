use parser::{
  internal::{ErrorNames, ErrorTypes},
  node_error,
};

use crate::{
  runtime::{
    binary_operation_error, delete_property_error,
    env::{RefEnvironment, TRUE_KEYWORD},
    get_instance_property_error, get_property_error, set_property_error,
    unary_back_operation_error, unary_operation_error, AgalArray, AgalBoolean, AgalByte,
    AgalComplex, AgalNumber, AgalPrimitive, AgalString, AgalThrow, AgalValuable,
    AgalValuableManager, AgalValue, RefAgalValue, Stack,
  },
  Modules,
};

#[derive(Clone, PartialEq)]
pub struct AgalError {
  type_error: ErrorNames,
  message: String,
  stack: Box<Stack>,
}
impl AgalError {
  pub fn new(type_error: ErrorNames, message: String, stack: Box<Stack>) -> AgalError {
    AgalError {
      type_error,
      message,
      stack,
    }
  }
  pub fn get_type_error(&self) -> ErrorNames {
    self.type_error.clone()
  }
  pub fn get_message(&self) -> String {
    self.message.clone()
  }
  pub fn to_error(&self) -> ErrorTypes {
    let value = self.stack.get_value();
    let error = if value.is_error() {
      value.get_error().unwrap().clone()
    } else {
      return ErrorTypes::StringError(format!("{}", self.get_message()));
    };
    node_error(&error)
  }
}
impl<'a> AgalValuable<'a> for AgalError {
  fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string(
      format!("{}: {}", self.type_error, self.message).as_str(),
    ))
  }
  fn to_agal_value(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string(
      format!("\x1b[91m{}\x1b[39m: {}", self.type_error, self.message).as_str(),
    ))
  }
  fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    let error = format!("\x1b[91m{}\x1b[39m: {}", self.type_error, self.message);
    let mut stack = String::new();
    let stack_vec = self.stack.iter();
    for (i, frame) in stack_vec.iter().enumerate() {
      stack.push_str(&format!(
        "{}:{}",
        frame.get_file(),
        frame.get_location().start.line
      ));
      if i < stack_vec.len() - 1 {
        stack.push_str(" -> ");
      }
    }
    let stack = format!("\x1b[90m{}\x1b[39m", stack);
    Ok(AgalString::from_string(format!("{} {}", error, stack)))
  }
  fn to_value(self) -> AgalValue<'a> {
    AgalComplex::Error(self).to_value()
  }
  fn get_instance_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    let value = self.to_value();
    get_instance_property_error(stack, env, key, &value)
  }

  fn get_keys(self) -> Vec<String> {
    std::vec![]
  }

  fn get_length(self) -> usize {
    0
  }

  fn to_agal_number(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::CustomError("Error Parseo"),
      message: "No se pudo convertir en numero".to_string(),
      stack: Box::new(stack.clone()),
    })
  }

  fn to_agal_boolean(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
    let value = env.as_ref().borrow();
    let value = value.get(stack, TRUE_KEYWORD, stack.get_value());
    let value: &AgalValue = &value.as_ref().borrow();
    match value {
      AgalValue::Primitive(AgalPrimitive::Boolean(b)) => Ok(b.clone()),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en booleano".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }

  fn to_agal_array(&self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::CustomError("Error Iterable"),
      message: "El valor no es iterable".to_string(),
      stack: Box::new(stack.clone()),
    })
  }

  fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::TypeError,
      message: "El valor no es un byte".to_string(),
      stack: Box::new(stack.clone()),
    })
  }

  fn binary_operation(
    &self,
    stack: &Stack,
    _env: RefEnvironment,
    operator: &str,
    other: RefAgalValue,
  ) -> RefAgalValue {
    binary_operation_error(stack, operator, self.clone().to_ref_value(), other)
  }

  fn unary_operator(&self, stack: &Stack, _env: RefEnvironment, operator: &str) -> RefAgalValue {
    unary_operation_error(stack, operator, self.clone().to_ref_value())
  }

  fn unary_back_operator(
    &self,
    stack: &Stack,
    _env: RefEnvironment,
    operator: &str,
  ) -> RefAgalValue {
    unary_back_operation_error(stack, operator, self.clone().to_ref_value())
  }

  fn get_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    get_property_error(stack, env, key)
  }

  fn set_object_property(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    key: String,
    _value: RefAgalValue,
  ) -> RefAgalValue {
    set_property_error(stack, env, key, "No se puede asignar".to_string())
  }

  fn delete_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) {
    delete_property_error(stack, env, key);
  }

  async fn call(
    &self,
    _: &Stack,
    _: RefEnvironment,
    _: RefAgalValue<'a>,
    _: Vec<RefAgalValue<'a>>,
    _: &Modules,
  ) -> RefAgalValue {
    AgalValue::Never.as_ref()
  }
}
