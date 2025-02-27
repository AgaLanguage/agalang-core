use crate::{libraries, runtime};

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
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_string(stack, modules),
      Self::Function(value) => value.to_agal_string(stack, modules),
      Self::Promise(value) => value.to_agal_string(stack, modules),
      Self::Object(value) => value.to_agal_string(stack, modules),
      Self::Array(value) => value.to_agal_string(stack, modules),
      Self::Class(value) => value.to_agal_string(stack, modules),
    }
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_console(stack, modules),
      Self::Function(value) => value.to_agal_console(stack, modules),
      Self::Promise(value) => value.to_agal_console(stack, modules),
      Self::Object(value) => value.to_agal_console(stack, modules),
      Self::Array(value) => value.to_agal_console(stack, modules),
      Self::Class(value) => value.to_agal_console(stack, modules),
    }
  }
  fn get_object_property(
    &self,
    stack: runtime::RefStack,
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
    stack: runtime::RefStack,
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
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.get_instance_property(stack, key, modules),
      Self::Function(value) => value.get_instance_property(stack, key, modules),
      Self::Promise(value) => value.get_instance_property(stack, key, modules),
      Self::Object(value) => value.get_instance_property(stack, key, modules),
      Self::Array(value) => value.get_instance_property(stack, key, modules),
      Self::Class(value) => value.get_instance_property(stack, key, modules),
    }
  }
  fn call(
    &self,
    stack: runtime::RefStack,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.call(stack, this, args, modules),
      Self::Function(value) => value.call(stack, this, args, modules),
      Self::Promise(value) => value.call(stack, this, args, modules),
      Self::Object(value) => value.call(stack, this, args, modules),
      Self::Array(value) => value.call(stack, this, args, modules),
      Self::Class(value) => value.call(stack, this, args, modules),
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
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_byte(stack, modules),
      Self::Function(value) => value.to_agal_byte(stack, modules),
      Self::Promise(value) => value.to_agal_byte(stack, modules),
      Self::Object(value) => value.to_agal_byte(stack, modules),
      Self::Array(value) => value.to_agal_byte(stack, modules),
      Self::Class(value) => value.to_agal_byte(stack, modules),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_boolean(stack, modules),
      Self::Function(value) => value.to_agal_boolean(stack, modules),
      Self::Promise(value) => value.to_agal_boolean(stack, modules),
      Self::Object(value) => value.to_agal_boolean(stack, modules),
      Self::Array(value) => value.to_agal_boolean(stack, modules),
      Self::Class(value) => value.to_agal_boolean(stack, modules),
    }
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::RefAgalValue<AgalArray>, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_array(stack, modules),
      Self::Function(value) => value.to_agal_array(stack, modules),
      Self::Promise(value) => value.to_agal_array(stack, modules),
      Self::Object(value) => value.to_agal_array(stack, modules),
      Self::Array(value) => value.to_agal_array(stack, modules),
      Self::Class(value) => value.to_agal_array(stack, modules),
    }
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: crate::parser::NodeOperator,
    right: super::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.binary_operation(stack, operator, right, modules),
      Self::Function(value) => value.binary_operation(stack, operator, right, modules),
      Self::Promise(value) => value.binary_operation(stack, operator, right, modules),
      Self::Object(value) => value.binary_operation(stack, operator, right, modules),
      Self::Array(value) => value.binary_operation(stack, operator, right, modules),
      Self::Class(value) => value.binary_operation(stack, operator, right, modules),
    }
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_number(stack, modules),
      Self::Function(value) => value.to_agal_number(stack, modules),
      Self::Promise(value) => value.to_agal_number(stack, modules),
      Self::Object(value) => value.to_agal_number(stack, modules),
      Self::Array(value) => value.to_agal_number(stack, modules),
      Self::Class(value) => value.to_agal_number(stack, modules),
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
