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

//#[derive(Clone)]
pub struct Promise(
  Pin<Box<dyn Future<Output = Result<values::DefaultRefAgalValue, internal::AgalThrow>>>>,
);
impl Promise {
  pub fn new(
    future: Pin<Box<dyn Future<Output = Result<values::DefaultRefAgalValue, internal::AgalThrow>>>>,
  ) -> Self {
    Self(future)
  }
}
impl Future for Promise {
  type Output = Result<values::DefaultRefAgalValue, internal::AgalThrow>;
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match self.as_mut().poll(cx) {
      Poll::Ready(value) => Poll::Ready(value),
      Poll::Pending => Poll::Pending,
    }
  }
}

pub enum AgalPromise {
  Unresolved(Promise),
  Resolved(Result<values::DefaultRefAgalValue, internal::AgalThrow>),
}
impl AgalPromise {
  pub fn new(inner: Promise) -> Self {
    Self::Unresolved(inner)
  }
}
impl IntoFuture for AgalPromise {
  type Output = Result<values::DefaultRefAgalValue, internal::AgalThrow>;
  type IntoFuture = Promise;

  fn into_future(self) -> Self::IntoFuture {
    match self {
      AgalPromise::Unresolved(inner) => inner,
      AgalPromise::Resolved(value) => Promise::new(Box::pin(async move { value })),
    }
  }
}
impl traits::ToAgalValue for AgalPromise {
  fn to_value(self) -> AgalValue {
    AgalComplex::Promise(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalPromise {
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string("Promise".to_string()))
  }
}
