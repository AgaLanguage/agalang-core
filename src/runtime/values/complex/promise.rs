use std::{
  borrow::BorrowMut,
  cell::RefCell,
  future::{self, Future, IntoFuture},
  ops::{Deref, DerefMut},
  pin::Pin,
  rc::Rc,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

use futures_util::FutureExt as _;
use parser::util::RefValue;
use tokio::task::JoinHandle;

use crate::runtime::values::{
  self, internal, primitive,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
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
    "Promesa".to_string()
  }
  fn to_agal_string(&self,stack: crate::runtime::RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string("Promise".to_string()))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &mut self,
    stack: crate::runtime::RefStack,
    env: crate::runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
  
  fn equals(&self, other: &Self) -> bool {
        todo!()
    }
  
  fn less_than(&self, other: &Self) -> bool {
        todo!()
    }
}
