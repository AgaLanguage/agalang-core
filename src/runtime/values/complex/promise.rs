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
  libraries,
  runtime::{
    self,
    values::{
      self, internal, primitive,
      traits::{self, AgalValuable, ToAgalValue as _},
      AgalValue,
    },
  },
};

use super::AgalComplex;

type Resolver = dyn FnOnce(values::DefaultRefAgalValue);
type Callback = Box<dyn FnOnce(Resolver, Resolver)>;

type ResultFuture = Result<values::DefaultRefAgalValue, internal::AgalThrow>;
pub enum AgalPromiseData {
  Unresolved(Pin<Box<dyn Future<Output = ResultFuture> + 'static>>),
  Resolved(ResultFuture),
}
impl AgalPromiseData {
  pub fn new(inner: Pin<Box<dyn Future<Output = ResultFuture> + 'static>>) -> Self {
    Self::Unresolved(inner)
  }
}
impl IntoFuture for AgalPromiseData {
  type Output = ResultFuture;
  type IntoFuture = Pin<Box<dyn Future<Output = ResultFuture> + 'static>>;

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
  pub fn new(inner: Pin<Box<dyn Future<Output = ResultFuture> + 'static>>) -> Self {
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
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    false
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
