use std::{
  borrow::BorrowMut,
  cell::RefCell,
  fmt::format,
  future::{self, Future, IntoFuture},
  ops::{Deref, DerefMut},
  pin::Pin,
  rc::Rc,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

use tokio::task::JoinHandle;

use crate::{
  functions_names, libraries, parser,
  runtime::{
    self,
    values::{
      self, error_message, internal, primitive,
      traits::{self, AgalValuable, ToAgalValue as _},
      AgalValue,
    },
  },
};
pub const PROMISE_THEN: &str = "luego";
pub const PROMISE_CATCH: &str = "atrapa";
use super::AgalComplex;

type Resolver = dyn FnOnce(values::DefaultRefAgalValue);
type Callback = Box<dyn FnOnce(Resolver, Resolver)>;

type ResultFuture = Result<values::DefaultRefAgalValue, internal::AgalThrow>;
pub enum AgalPromiseData {
  Unresolved(Pin<Box<dyn Future<Output = ResultFuture>>>),
  Resolved(ResultFuture),
}
impl AgalPromiseData {
  pub fn new(inner: Pin<Box<dyn Future<Output = ResultFuture>>>) -> Self {
    Self::Unresolved(inner)
  }
}
impl IntoFuture for AgalPromiseData {
  type Output = ResultFuture;
  type IntoFuture = Pin<Box<dyn Future<Output = ResultFuture>>>;

  fn into_future(self) -> Self::IntoFuture {
    match self {
      Self::Unresolved(inner) => inner,
      Self::Resolved(value) => Box::pin(async move { value }),
    }
  }
}
impl Default for AgalPromiseData {
  fn default() -> Self {
    Self::Resolved(AgalValue::Never.to_result())
  }
}

pub struct AgalPromise {
  pub data: AgalPromiseData,
}
impl std::fmt::Debug for AgalPromise {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "AgalPromise")
  }
}
impl AgalPromise {
  pub fn new(inner: Pin<Box<dyn Future<Output = ResultFuture>>>) -> Self {
    Self {
      data: AgalPromiseData::Unresolved(inner),
    }
  }
}
impl traits::ToAgalValue for AgalPromise {
  fn to_value(self) -> AgalValue {
    AgalComplex::Promise(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalPromise {
  fn get_name(&self) -> String {
    match &self.data {
      AgalPromiseData::Unresolved(_) => "Promesa".to_string(),
      AgalPromiseData::Resolved(value) => format!(
        "{}",
        match value {
          Ok(value) => value.get_name(),
          Err(value) => value.get_name(),
        }
      ),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(primitive::AgalBoolean::True)
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      functions_names::TO_AGAL_STRING | PROMISE_THEN | PROMISE_CATCH => modules
        .get_module(":proto/Promesa")
        .ok_or_else(|| internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
          stack: stack.clone(),
        })?
        .get_instance_property(stack, key, modules),
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    false
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
