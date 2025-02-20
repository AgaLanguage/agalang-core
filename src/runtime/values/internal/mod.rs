use super::{
  primitive,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};
use crate::runtime::{self, values::internal};

mod native_function;
pub use native_function::*;
mod throw;
pub use throw::*;
mod error;
pub use error::*;
mod lazy;
pub use lazy::*;

#[derive(Clone, Debug)]
pub enum AgalInternal {
  Lazy(AgalLazy),
  Throw(AgalThrow),
  Error(AgalError),
  Return(super::DefaultRefAgalValue),
  NativeFunction(native_function::AgalNativeFunction),
}
impl AgalInternal {
  pub fn is_return(&self) -> bool {
    match self {
      AgalInternal::Return(_) => true,
      _ => false,
    }
  }
  pub fn into_return(self) -> Option<super::DefaultRefAgalValue> {
    match self {
      AgalInternal::Return(val) => Some(val),
      _ => None,
    }
  }
}
impl traits::ToAgalValue for AgalInternal {
  fn to_value(self) -> AgalValue {
    AgalValue::Internal(self.as_ref())
  }
  fn to_result(self) -> Result<super::DefaultRefAgalValue, AgalThrow>
  where
    Self: Sized,
  {
    match self {
      Self::Throw(a) => Err(a),
      _ => Ok(self.to_ref_value()),
    }
  }
}
impl traits::AgalValuable for AgalInternal {
  fn get_name(&self) -> String {
    match self {
      Self::Lazy(lazy) => lazy.get_name(),
      Self::Throw(throw) => throw.get_name(),
      Self::Error(error) => error.get_name(),
      Self::Return(value) => value.get_name(),
      Self::NativeFunction(func) => func.get_name(),
    }
  }
  fn to_agal_string(&self, stack: runtime::RefStack) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_string(stack),
      Self::Throw(throw) => throw.to_agal_string(stack),
      Self::Error(error) => error.to_agal_string(stack),
      Self::Return(val) => val.to_agal_string(stack),
      Self::NativeFunction(func) => func.to_agal_string(stack),
    }
  }
  fn to_agal_console(&self, stack: runtime::RefStack) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_console(stack),
      Self::Throw(throw) => throw.to_agal_console(stack),
      Self::Error(error) => error.to_agal_console(stack),
      Self::Return(val) => val.to_agal_console(stack),
      Self::NativeFunction(func) => func.to_agal_console(stack),
    }
  }
  async fn call(
    &mut self,
    stack: runtime::RefStack,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.call(stack, this, args, modules).await,
      Self::Throw(throw) => throw.call(stack, this, args, modules).await,
      Self::Error(error) => error.call(stack, this, args, modules).await,
      Self::Return(val) => val.call(stack, this, args, modules).await,
      Self::NativeFunction(func) => func.call(stack, this, args, modules).await,
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Lazy(l) => l.get_keys(),
      Self::Throw(t) => t.get_keys(),
      Self::Error(e) => e.get_keys(),
      Self::Return(r) => r.get_keys(),
      Self::NativeFunction(f) => f.get_keys(),
    }
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_byte(stack),
      Self::Throw(t) => t.to_agal_byte(stack),
      Self::Lazy(l) => l.to_agal_byte(stack),
      Self::Return(r) => r.to_agal_byte(stack),
      Self::NativeFunction(f) => f.to_agal_byte(stack),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_boolean(stack),
      Self::Throw(t) => t.to_agal_boolean(stack),
      Self::Lazy(l) => l.to_agal_boolean(stack),
      Self::Return(r) => r.to_agal_boolean(stack),
      Self::NativeFunction(f) => f.to_agal_boolean(stack),
    }
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<super::RefAgalValue<super::complex::AgalArray>, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_array(stack),
      Self::Throw(t) => t.to_agal_array(stack),
      Self::Lazy(l) => l.to_agal_array(stack),
      Self::Return(r) => r.to_agal_array(stack),
      Self::NativeFunction(f) => f.to_agal_array(stack),
    }
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: &str,
    right: super::DefaultRefAgalValue,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.binary_operation(stack, operator, right),
      Self::Throw(t) => t.binary_operation(stack, operator, right),
      Self::Lazy(l) => l.binary_operation(stack, operator, right),
      Self::Return(r) => r.binary_operation(stack, operator, right),
      Self::NativeFunction(f) => f.binary_operation(stack, operator, right),
    }
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    operator: &str,
  ) -> super::ResultAgalValue {
    match self {
      Self::Error(e) => e.unary_back_operator(stack, operator),
      Self::Throw(t) => t.unary_back_operator(stack, operator),
      Self::Lazy(l) => l.unary_back_operator(stack, operator),
      Self::Return(r) => r.unary_back_operator(stack, operator),
      Self::NativeFunction(f) => f.unary_back_operator(stack, operator),
    }
  }

  fn unary_operator(&self, stack: runtime::RefStack, operator: &str) -> super::ResultAgalValue {
    match self {
      Self::Error(e) => e.unary_operator(stack, operator),
      Self::Throw(t) => t.unary_operator(stack, operator),
      Self::Lazy(l) => l.unary_operator(stack, operator),
      Self::Return(r) => r.unary_operator(stack, operator),
      Self::NativeFunction(f) => f.unary_operator(stack, operator),
    }
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.get_object_property(stack, key),
      Self::Throw(t) => t.get_object_property(stack, key),
      Self::Lazy(l) => l.get_object_property(stack, key),
      Self::Return(r) => r.get_object_property(stack, key),
      Self::NativeFunction(f) => f.get_object_property(stack, key),
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
      Self::Throw(t) => t.set_object_property(stack, key, value),
      Self::Lazy(l) => l.set_object_property(stack, key, value),
      Self::Return(r) => r.set_object_property(stack, key, value),
      Self::NativeFunction(f) => f.set_object_property(stack, key, value),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.get_instance_property(stack, key),
      Self::Throw(t) => t.get_instance_property(stack, key),
      Self::Lazy(l) => l.get_instance_property(stack, key),
      Self::Return(r) => r.get_instance_property(stack, key),
      Self::NativeFunction(f) => f.get_instance_property(stack, key),
    }
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::Error(e) => e.to_agal_number(stack),
      Self::Throw(t) => t.to_agal_number(stack),
      Self::Lazy(l) => l.to_agal_number(stack),
      Self::Return(r) => r.to_agal_number(stack),
      Self::NativeFunction(f) => f.to_agal_number(stack),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Error(e1), Self::Error(e2)) => e1.equals(e2),
      (Self::Throw(t1), Self::Throw(t2)) => t1.equals(t2),
      (Self::Lazy(l1), Self::Lazy(l2)) => l1.equals(l2),
      (Self::Return(r1), Self::Return(r2)) => r1.equals(r2),
      (Self::NativeFunction(f1), Self::NativeFunction(f2)) => f1.equals(f2),
      _ => false,
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Error(e1), Self::Error(e2)) => e1.less_than(e2),
      (Self::Throw(t1), Self::Throw(t2)) => t1.less_than(t2),
      (Self::Lazy(l1), Self::Lazy(l2)) => l1.less_than(l2),
      (Self::Return(r1), Self::Return(r2)) => r1.less_than(r2),
      (Self::NativeFunction(f1), Self::NativeFunction(f2)) => f1.less_than(f2),
      _ => false,
    }
  }
}
