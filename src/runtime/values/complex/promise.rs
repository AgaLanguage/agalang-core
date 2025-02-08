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

//#[derive(Clone)]
pub struct Promise(
  Pin<Box<dyn Future<Output = ResultFuture>>>,
);
impl Promise {
  pub fn new(
    future: Pin<Box<dyn Future<Output = ResultFuture>>>,
  ) -> Self {
    Self(future)
  }
}
impl Future for Promise {
  type Output = ResultFuture;
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.get_mut().0.as_mut().poll(cx)
  }
}

pub enum AgalPromiseData {
  Unresolved(Promise),
  Resolved(ResultFuture),
}
impl AgalPromiseData {
  pub fn new(inner: Promise) -> Self {
    Self::Unresolved(inner)
  }
}
impl IntoFuture for AgalPromiseData {
  type Output = ResultFuture;
  type IntoFuture = Promise;

  fn into_future(self) -> Self::IntoFuture {
    match self {
      Self::Unresolved(inner) => inner,
      Self::Resolved(value) => Promise::new(Box::pin(async move { value })),
    }
  }
}

pub struct AgalPromise {
  pub data: AgalPromiseData
}
impl AgalPromise {
  pub fn new(inner: Promise) -> Self {
    Self { data: AgalPromiseData::Unresolved(inner) }
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
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string("Promise".to_string()))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
}
