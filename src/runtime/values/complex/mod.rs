use parser::util::RefValue;

use crate::{
  runtime::{env::RefEnvironment, stack::Stack},
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

#[derive(Clone)]
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
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_string(),
      Self::Function(value) => value.to_agal_string(),
      Self::Promise(value) => value.to_agal_string(),
      Self::Object(value) => value.to_agal_string(),
      Self::Array(value) => value.to_agal_string(),
      Self::Class(value) => value.to_agal_string(),
    }
  }
  fn to_agal_console(
    &self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.to_agal_console(stack, env),
      Self::Function(value) => value.to_agal_console(stack, env),
      Self::Promise(value) => value.to_agal_console(stack, env),
      Self::Object(value) => value.to_agal_console(stack, env),
      Self::Array(value) => value.to_agal_console(stack, env),
      Self::Class(value) => value.to_agal_console(stack, env),
    }
  }
  fn get_object_property(
    &self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.get_object_property(stack, env, key),
      Self::Function(value) => value.get_object_property(stack, env, key),
      Self::Promise(value) => value.get_object_property(stack, env, key),
      Self::Object(value) => value.get_object_property(stack, env, key),
      Self::Array(value) => value.get_object_property(stack, env, key),
      Self::Class(value) => value.get_object_property(stack, env, key),
    }
  }
  fn set_object_property(
    &mut self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    key: &str,
    value: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(val) => val.set_object_property(stack, env, key, value),
      Self::Function(val) => val.set_object_property(stack, env, key, value),
      Self::Promise(val) => val.set_object_property(stack, env, key, value),
      Self::Object(val) => val.set_object_property(stack, env, key, value),
      Self::Array(val) => val.set_object_property(stack, env, key, value),
      Self::Class(val) => val.set_object_property(stack, env, key, value),
    }
  }
  fn get_instance_property(
    &self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.get_instance_property(stack, env, key),
      Self::Function(value) => value.get_instance_property(stack, env, key),
      Self::Promise(value) => value.get_instance_property(stack, env, key),
      Self::Object(value) => value.get_instance_property(stack, env, key),
      Self::Array(value) => value.get_instance_property(stack, env, key),
      Self::Class(value) => value.get_instance_property(stack, env, key),
    }
  }
  async fn call(
    &self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::SuperInstance(value) => value.call(stack, env, this, args, modules).await,
      Self::Function(value) => value.call(stack, env, this, args, modules).await,
      Self::Promise(value) => value.call(stack, env, this, args, modules).await,
      Self::Object(value) => value.call(stack, env, this, args, modules).await,
      Self::Array(value) => value.call(stack, env, this, args, modules).await,
      Self::Class(value) => value.call(stack, env, this, args, modules).await,
    }
  }
}
