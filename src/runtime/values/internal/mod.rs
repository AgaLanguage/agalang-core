use super::{
  primitive,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};
use crate::{
  libraries,
  runtime::{self, values::internal},
};

mod native_function;
pub use native_function::*;
mod throw;
pub use throw::*;
mod error;
pub use error::*;
mod lazy;
pub use lazy::*;
mod immutable;
pub use immutable::*;

#[derive(Clone, Debug)]
pub enum AgalInternal {
  Lazy(AgalLazy),
  Error(AgalError),
  Immutable(AgalImmutable),
  NativeFunction(native_function::AgalNativeFunction),
}
impl traits::ToAgalValue for AgalInternal {
  fn to_value(self) -> AgalValue {
    AgalValue::Internal(self.as_ref())
  }
}
impl traits::AgalValuable for AgalInternal {
  fn get_name(&self) -> String {
    match self {
      Self::Lazy(lazy) => lazy.get_name(),
      Self::Error(error) => error.get_name(),
      Self::NativeFunction(func) => func.get_name(),
      Self::Immutable(immutable) => immutable.get_name(),
    }
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_string(stack, modules),
      Self::Error(error) => error.to_agal_string(stack, modules),
      Self::NativeFunction(func) => func.to_agal_string(stack, modules),
      Self::Immutable(immutable) => immutable.to_agal_string(stack, modules),
    }
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_console(stack, modules),
      Self::Error(error) => error.to_agal_console(stack, modules),
      Self::NativeFunction(func) => func.to_agal_console(stack, modules),
      Self::Immutable(immutable) => immutable.to_agal_console(stack, modules),
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
      Self::Lazy(lazy) => lazy.call(stack, this, args, modules),
      Self::Error(error) => error.call(stack, this, args, modules),
      Self::NativeFunction(func) => func.call(stack, this, args, modules),
      Self::Immutable(immutable) => immutable.call(stack, this, args, modules),
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Lazy(l) => l.get_keys(),
      Self::Error(e) => e.get_keys(),
      Self::NativeFunction(f) => f.get_keys(),
      Self::Immutable(i) => i.get_keys(),
    }
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_byte(stack, modules),
      Self::Lazy(l) => l.to_agal_byte(stack, modules),
      Self::NativeFunction(f) => f.to_agal_byte(stack, modules),
      Self::Immutable(i) => i.to_agal_byte(stack, modules),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_boolean(stack, modules),
      Self::Lazy(l) => l.to_agal_boolean(stack, modules),
      Self::NativeFunction(f) => f.to_agal_boolean(stack, modules),
      Self::Immutable(i) => i.to_agal_boolean(stack, modules),
    }
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::RefAgalValue<super::complex::AgalArray>, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_array(stack, modules),
      Self::Lazy(l) => l.to_agal_array(stack, modules),
      Self::NativeFunction(f) => f.to_agal_array(stack, modules),
      Self::Immutable(i) => i.to_agal_array(stack, modules),
    }
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: crate::parser::NodeOperator,
    right: super::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.binary_operation(stack, operator, right, modules),
      Self::Lazy(l) => l.binary_operation(stack, operator, right, modules),
      Self::NativeFunction(f) => f.binary_operation(stack, operator, right, modules),
      Self::Immutable(i) => i.binary_operation(stack, operator, right, modules),
    }
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.get_object_property(stack, key),
      Self::Lazy(l) => l.get_object_property(stack, key),
      Self::NativeFunction(f) => f.get_object_property(stack, key),
      Self::Immutable(i) => i.get_object_property(stack, key),
    }
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.set_object_property(stack, key, value),
      Self::Lazy(l) => l.set_object_property(stack, key, value),
      Self::NativeFunction(f) => f.set_object_property(stack, key, value),
      Self::Immutable(i) => i.set_object_property(stack, key, value),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.get_instance_property(stack, key, modules),
      Self::Lazy(l) => l.get_instance_property(stack, key, modules),
      Self::NativeFunction(f) => f.get_instance_property(stack, key, modules),
      Self::Immutable(i) => i.get_instance_property(stack, key, modules),
    }
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_number(stack, modules),
      Self::Lazy(l) => l.to_agal_number(stack, modules),
      Self::NativeFunction(f) => f.to_agal_number(stack, modules),
      Self::Immutable(i) => i.to_agal_number(stack, modules),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Error(e1), Self::Error(e2)) => e1.equals(e2),
      (Self::Lazy(l1), Self::Lazy(l2)) => l1.equals(l2),
      (Self::NativeFunction(f1), Self::NativeFunction(f2)) => f1.equals(f2),
      (Self::Immutable(i1), Self::Immutable(i2)) => i1.equals(i2),
      (Self::Immutable(i), o) => i.get_value().equals(&o.clone().to_ref_value()),
      (o, Self::Immutable(i)) => o.clone().to_ref_value().equals(&i.get_value()),
      _ => false,
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Error(e1), Self::Error(e2)) => e1.less_than(e2),
      (Self::Lazy(l1), Self::Lazy(l2)) => l1.less_than(l2),
      (Self::NativeFunction(f1), Self::NativeFunction(f2)) => f1.less_than(f2),
      (Self::Immutable(i1), Self::Immutable(i2)) => i1.less_than(i2),
      (Self::Immutable(i), o) => i.get_value().less_than(&o.clone().to_ref_value()),
      (o, Self::Immutable(i)) => o.clone().to_ref_value().less_than(&i.get_value()),
      _ => false,
    }
  }
}
