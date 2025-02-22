use crate::{libraries, runtime};

use super::{
  internal,
  traits::{self, AgalValuable},
  AgalValue,
};

mod boolean;
pub use boolean::*;
mod byte;
pub use byte::*;
mod number;
pub use number::*;
mod string;
pub use string::*;
#[derive(Clone, Debug)]
pub enum AgalPrimitive {
  Boolean(boolean::AgalBoolean),
  Number(number::AgalNumber),
  String(string::AgalString),
  Char(string::AgalChar),
  Byte(byte::AgalByte),
}
impl traits::ToAgalValue for AgalPrimitive {
  fn to_value(self) -> AgalValue {
    AgalValue::Primitive(self.as_ref())
  }
}
impl traits::AgalValuable for AgalPrimitive {
  fn get_name(&self) -> String {
    match self {
      Self::Boolean(b) => b.get_name(),
      Self::Number(n) => n.get_name(),
      Self::String(s) => s.get_name(),
      Self::Char(c) => c.get_name(),
      Self::Byte(b) => b.get_name(),
    }
  }
  fn to_agal_number(&self, stack: runtime::RefStack) -> Result<AgalNumber, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_number(stack),
      Self::Number(value) => value.to_agal_number(stack),
      Self::String(value) => value.to_agal_number(stack),
      Self::Char(value) => value.to_agal_number(stack),
      Self::Byte(value) => value.to_agal_number(stack),
    }
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
  ) -> Result<string::AgalString, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_string(stack),
      Self::Number(value) => value.to_agal_string(stack),
      Self::String(value) => value.to_agal_string(stack),
      Self::Char(value) => value.to_agal_string(stack),
      Self::Byte(value) => value.to_agal_string(stack),
    }
  }
  fn to_agal_byte(&self, stack: runtime::RefStack) -> Result<byte::AgalByte, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_byte(stack),
      Self::Number(value) => value.to_agal_byte(stack),
      Self::String(value) => value.to_agal_byte(stack),
      Self::Char(value) => value.to_agal_byte(stack),
      Self::Byte(value) => value.to_agal_byte(stack),
    }
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
  ) -> Result<string::AgalString, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_console(stack),
      Self::Number(value) => value.to_agal_console(stack),
      Self::String(value) => value.to_agal_console(stack),
      Self::Char(value) => value.to_agal_console(stack),
      Self::Byte(value) => value.to_agal_console(stack),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<boolean::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => Ok(*value),
      Self::Number(value) => value.to_agal_boolean(stack),
      Self::String(value) => value.to_agal_boolean(stack),
      Self::Char(value) => value.to_agal_boolean(stack),
      Self::Byte(value) => value.to_agal_boolean(stack),
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Boolean(b) => b.get_keys(),
      Self::Number(n) => n.get_keys(),
      Self::String(s) => s.get_keys(),
      Self::Char(c) => c.get_keys(),
      Self::Byte(b) => b.get_keys(),
    }
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<super::RefAgalValue<super::complex::AgalArray>, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.to_agal_array(stack),
      Self::Number(n) => n.to_agal_array(stack),
      Self::String(s) => s.to_agal_array(stack),
      Self::Char(c) => c.to_agal_array(stack),
      Self::Byte(b) => b.to_agal_array(stack),
    }
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.binary_operation(stack, operator, right),
      Self::Number(n) => n.binary_operation(stack, operator, right),
      Self::String(s) => s.binary_operation(stack, operator, right),
      Self::Char(c) => c.binary_operation(stack, operator, right),
      Self::Byte(b) => b.binary_operation(stack, operator, right),
    }
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.get_object_property(stack, key),
      Self::Number(n) => n.get_object_property(stack, key),
      Self::String(s) => s.get_object_property(stack, key),
      Self::Char(c) => c.get_object_property(stack, key),
      Self::Byte(b) => b.get_object_property(stack, key),
    }
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.set_object_property(stack, key, value),
      Self::Number(n) => n.set_object_property(stack, key, value),
      Self::String(s) => s.set_object_property(stack, key, value),
      Self::Char(c) => c.set_object_property(stack, key, value),
      Self::Byte(b) => b.set_object_property(stack, key, value),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.get_instance_property(stack, key,modules),
      Self::Number(n) => n.get_instance_property(stack, key,modules),
      Self::String(s) => s.get_instance_property(stack, key,modules),
      Self::Char(c) => c.get_instance_property(stack, key,modules),
      Self::Byte(b) => b.get_instance_property(stack, key,modules),
    }
  }

  async fn call(
    &self,
    stack: runtime::RefStack,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Boolean(b) => b.call(stack, this, args, modules).await,
      Self::Number(n) => n.call(stack, this, args, modules).await,
      Self::String(s) => s.call(stack, this, args, modules).await,
      Self::Char(c) => c.call(stack, this, args, modules).await,
      Self::Byte(b) => b.call(stack, this, args, modules).await,
    }
  }

  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Boolean(b1), Self::Boolean(b2)) => b1 == b2,
      (Self::Number(n1), Self::Number(n2)) => n1 == n2,
      (Self::String(s1), Self::String(s2)) => s1 == s2,
      (Self::Char(c1), Self::Char(c2)) => c1 == c2,
      (Self::Byte(b1), Self::Byte(b2)) => b1 == b2,
      _ => false,
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Boolean(b1), Self::Boolean(b2)) => b1 < b2,
      (Self::Number(n1), Self::Number(n2)) => n1 < n2,
      (Self::String(s1), Self::String(s2)) => s1 < s2,
      (Self::Char(c1), Self::Char(c2)) => c1 < c2,
      (Self::Byte(b1), Self::Byte(b2)) => b1 < b2,
      _ => false,
    }
  }
}
