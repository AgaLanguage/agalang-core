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

#[derive(Clone)]
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
  pub fn to_result(&self) -> super::ResultAgalValue {
    match self {
      AgalInternal::Throw(a) => Err(a.clone()),
      _ => Ok(self.clone().to_ref_value()),
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
  fn to_agal_string(&self) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_string(),
      Self::Throw(throw) => throw.to_agal_string(),
      Self::Error(error) => error.to_agal_string(),
      Self::Return(val) => val.to_agal_string(),
      Self::NativeFunction(func) => func.to_agal_string(),
    }
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.to_agal_console(stack, env),
      Self::Throw(throw) => throw.to_agal_console(stack, env),
      Self::Error(error) => error.to_agal_console(stack, env),
      Self::Return(val) => val.to_agal_console(stack, env),
      Self::NativeFunction(func) => func.to_agal_console(stack, env),
    }
  }
  async fn call(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: super::DefaultRefAgalValue,
    args: Vec<super::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<super::DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Lazy(lazy) => lazy.call(stack, env, this, args, modules).await,
      Self::Throw(throw) => throw.call(stack, env, this, args, modules).await,
      Self::Error(error) => error.call(stack, env, this, args, modules).await,
      Self::Return(val) => val.call(stack, env, this, args, modules).await,
      Self::NativeFunction(func) => func.call(stack, env, this, args, modules).await,
    }
  }
}
