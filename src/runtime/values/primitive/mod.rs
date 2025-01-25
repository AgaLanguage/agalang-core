use parser::util;

use crate::runtime;

use super::{
  internal,
  traits::{self, AgalValuable as _},
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
  fn to_agal_string(&self) -> Result<string::AgalString, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_string(),
      Self::Number(value) => value.to_agal_string(),
      Self::String(value) => value.to_agal_string(),
      Self::Char(value) => value.to_agal_string(),
      Self::Byte(value) => value.to_agal_string(),
    }
  }
  fn to_agal_byte(
    &self,
    stack: util::RefValue<runtime::Stack>,
  ) -> Result<byte::AgalByte, internal::AgalThrow> {
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
    stack: util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<string::AgalString, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_console(stack, env),
      Self::Number(value) => value.to_agal_console(stack, env),
      Self::String(value) => value.to_agal_console(stack, env),
      Self::Char(value) => value.to_agal_console(stack, env),
      Self::Byte(value) => value.to_agal_console(stack, env),
    }
  }
  fn to_agal_value(
    &self,
    stack: util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<string::AgalString, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => value.to_agal_value(stack, env),
      Self::Number(value) => value.to_agal_value(stack, env),
      Self::String(value) => value.to_agal_value(stack, env),
      Self::Char(value) => value.to_agal_value(stack, env),
      Self::Byte(value) => value.to_agal_value(stack, env),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: util::RefValue<runtime::Stack>,
  ) -> Result<boolean::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Boolean(value) => Ok(*value),
      Self::Number(value) => value.to_agal_boolean(stack),
      Self::String(value) => value.to_agal_boolean(stack),
      Self::Char(value) => value.to_agal_boolean(stack),
      Self::Byte(value) => value.to_agal_boolean(stack),
    }
  }
}
