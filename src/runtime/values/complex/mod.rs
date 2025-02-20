use parser::util::RefValue;

use crate::{
  runtime::{env::RefEnvironment, stack::RefStack},
  Modules,
};

use super::{
  internal, primitive,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};

mod array;
pub use array::*;
mod class;
pub use class::*;
mod function;
pub use function::*;
mod object;
pub use object::*;
mod promise;
pub use promise::*;

#[derive(Clone, Debug)]
pub enum AgalComplex {
  SuperInstance(super::RefAgalValue<class::AgalPrototype>),
  Function(super::RefAgalValue<function::AgalFunction>),
  Promise(super::RefAgalValue<promise::AgalPromise>),
  Object(super::RefAgalValue<object::AgalObject>),
  Array(super::RefAgalValue<array::AgalArray>),
  Class(super::RefAgalValue<class::AgalClass>),
}
impl traits::ToAgalValue for AgalComplex {
  fn to_value(self) -> AgalValue {
    AgalValue::Complex(self.as_ref())
  }
}
impl traits::AgalValuable for AgalComplex {
  fn get_name(&self) -> String {
    match self {
      Self::SuperInstance(value) => value.get_name(),
      Self::Function(value) => value.get_name(),
      Self::Promise(value) => value.get_name(),
      Self::Object(value) => value.get_name(),
      Self::Array(value) => value.get_name(),
      Self::Class(value) => value.get_name(),
    }
  }
  fn to_agal_string(&self, stack: RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_string(stack),
      Self::Function(value) => value.to_agal_string(stack),
      Self::Promise(value) => value.to_agal_string(stack),
      Self::Object(value) => value.to_agal_string(stack),
      Self::Array(value) => value.to_agal_string(stack),
      Self::Class(value) => value.to_agal_string(stack),
    }
  }
  fn to_agal_console(&self, stack: RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_console(stack),
      Self::Function(value) => value.to_agal_console(stack),
      Self::Promise(value) => value.to_agal_console(stack),
      Self::Object(value) => value.to_agal_console(stack),
      Self::Array(value) => value.to_agal_console(stack),
      Self::Class(value) => value.to_agal_console(stack),
    }
  }
  fn get_object_property(
    &self,
    stack: RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.get_object_property(stack, key),
      Self::Function(value) => value.get_object_property(stack, key),
      Self::Promise(value) => value.get_object_property(stack, key),
      Self::Object(value) => value.get_object_property(stack, key),
      Self::Array(value) => value.get_object_property(stack, key),
      Self::Class(value) => value.get_object_property(stack, key),
    }
  }
  fn set_object_property(
    &mut self,
    stack: RefStack,
    key: &str,
    value: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(val) => val.set_object_property(stack, key, value),
      Self::Function(val) => val.set_object_property(stack, key, value),
      Self::Promise(val) => val.set_object_property(stack, key, value),
      Self::Object(val) => val.set_object_property(stack, key, value),
      Self::Array(val) => val.set_object_property(stack, key, value),
      Self::Class(val) => val.set_object_property(stack, key, value),
    }
  }
  fn get_instance_property(
    &self,
    stack: RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.get_instance_property(stack, key),
      Self::Function(value) => value.get_instance_property(stack, key),
      Self::Promise(value) => value.get_instance_property(stack, key),
      Self::Object(value) => value.get_instance_property(stack, key),
      Self::Array(value) => value.get_instance_property(stack, key),
      Self::Class(value) => value.get_instance_property(stack, key),
    }
  }
  async fn call(
    &mut self,
    stack: RefStack,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.call(stack, this, args, modules).await,
      Self::Function(value) => value.call(stack, this, args, modules).await,
      Self::Promise(value) => value.call(stack, this, args, modules).await,
      Self::Object(value) => value.call(stack, this, args, modules).await,
      Self::Array(value) => value.call(stack, this, args, modules).await,
      Self::Class(value) => value.call(stack, this, args, modules).await,
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::SuperInstance(value) => value.get_keys(),
      Self::Function(value) => value.get_keys(),
      Self::Promise(value) => value.get_keys(),
      Self::Object(value) => value.get_keys(),
      Self::Array(value) => value.get_keys(),
      Self::Class(value) => value.get_keys(),
    }
  }

  fn to_agal_byte(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_byte(stack),
      Self::Function(value) => value.to_agal_byte(stack),
      Self::Promise(value) => value.to_agal_byte(stack),
      Self::Object(value) => value.to_agal_byte(stack),
      Self::Array(value) => value.to_agal_byte(stack),
      Self::Class(value) => value.to_agal_byte(stack),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_boolean(stack),
      Self::Function(value) => value.to_agal_boolean(stack),
      Self::Promise(value) => value.to_agal_boolean(stack),
      Self::Object(value) => value.to_agal_boolean(stack),
      Self::Array(value) => value.to_agal_boolean(stack),
      Self::Class(value) => value.to_agal_boolean(stack),
    }
  }

  fn to_agal_array(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<super::RefAgalValue<AgalArray>, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_array(stack),
      Self::Function(value) => value.to_agal_array(stack),
      Self::Promise(value) => value.to_agal_array(stack),
      Self::Object(value) => value.to_agal_array(stack),
      Self::Array(value) => value.to_agal_array(stack),
      Self::Class(value) => value.to_agal_array(stack),
    }
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
    right: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.binary_operation(stack, operator, right),
      Self::Function(value) => value.binary_operation(stack, operator, right),
      Self::Promise(value) => value.binary_operation(stack, operator, right),
      Self::Object(value) => value.binary_operation(stack, operator, right),
      Self::Array(value) => value.binary_operation(stack, operator, right),
      Self::Class(value) => value.binary_operation(stack, operator, right),
    }
  }

  fn unary_back_operator(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
  ) -> super::ResultAgalValue {
    match self {
      Self::SuperInstance(value) => value.unary_back_operator(stack, operator),
      Self::Function(value) => value.unary_back_operator(stack, operator),
      Self::Promise(value) => value.unary_back_operator(stack, operator),
      Self::Object(value) => value.unary_back_operator(stack, operator),
      Self::Array(value) => value.unary_back_operator(stack, operator),
      Self::Class(value) => value.unary_back_operator(stack, operator),
    }
  }

  fn unary_operator(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
  ) -> super::ResultAgalValue {
    match self {
      Self::SuperInstance(value) => value.unary_operator(stack, operator),
      Self::Function(value) => value.unary_operator(stack, operator),
      Self::Promise(value) => value.unary_operator(stack, operator),
      Self::Object(value) => value.unary_operator(stack, operator),
      Self::Array(value) => value.unary_operator(stack, operator),
      Self::Class(value) => value.unary_operator(stack, operator),
    }
  }

  fn to_agal_number(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_number(stack),
      Self::Function(value) => value.to_agal_number(stack),
      Self::Promise(value) => value.to_agal_number(stack),
      Self::Object(value) => value.to_agal_number(stack),
      Self::Array(value) => value.to_agal_number(stack),
      Self::Class(value) => value.to_agal_number(stack),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::SuperInstance(a), Self::SuperInstance(b)) => a.equals(b),
      (Self::Function(a), Self::Function(b)) => a.equals(b),
      (Self::Promise(a), Self::Promise(b)) => a.equals(b),
      (Self::Object(a), Self::Object(b)) => a.equals(b),
      (Self::Array(a), Self::Array(b)) => a.equals(b),
      (Self::Class(a), Self::Class(b)) => a.equals(b),
      _ => false,
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::SuperInstance(a), Self::SuperInstance(b)) => a.less_than(b),
      (Self::Function(a), Self::Function(b)) => a.less_than(b),
      (Self::Promise(a), Self::Promise(b)) => a.less_than(b),
      (Self::Object(a), Self::Object(b)) => a.less_than(b),
      (Self::Array(a), Self::Array(b)) => a.less_than(b),
      (Self::Class(a), Self::Class(b)) => a.less_than(b),
      _ => false,
    }
  }
}
