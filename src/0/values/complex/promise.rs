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

use crate::runtime::{AgalValuable, AgalValuableManager, AgalValue, RefAgalValue};

use super::AgalComplex;

type Resolver = dyn FnOnce(RefAgalValue);
type Callback = Box<dyn FnOnce(Resolver, Resolver)>;

//#[derive(Clone)]
pub struct Promise<'a>(Pin<Box<dyn Future<Output = Result<RefAgalValue<'a>, RefAgalValue<'a>>>>>);
impl<'a> Future for Promise<'a> {
  type Output = Result<RefAgalValue<'a>, RefAgalValue<'a>>;
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match self.as_mut().poll(cx) {
      Poll::Ready(value) => Poll::Ready(value),
      Poll::Pending => Poll::Pending,
    }
  }
}

pub struct AgalPromise<'a> {
  inner: Promise<'a>,
}

impl<'a> AgalPromise<'a> {
  pub fn new(inner: Promise<'a>) -> Self {
    Self { inner }
  }
}
impl<'a> IntoFuture for AgalPromise<'a> {
  type Output = Result<RefAgalValue<'a>, RefAgalValue<'a>>;
  type IntoFuture = Promise<'a>;

  fn into_future(self) -> Self::IntoFuture {
    self.inner
  }
}
impl<'a> AgalValuable<'a> for AgalPromise<'a> {
  fn to_value(self) -> AgalValue<'a> {
    AgalComplex::Promise(self).to_value()
  }

  fn to_agal_console(
    &self,
    stack: &crate::runtime::Stack,
    env: runtime::RefEnvironment,
  ) -> Result<crate::runtime::AgalString, crate::runtime::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: &crate::runtime::Stack,
    env: runtime::RefEnvironment,
    key: String,
  ) -> RefAgalValue {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: &crate::runtime::Stack,
    env: runtime::RefEnvironment,
    operator: &str,
    other: RefAgalValue<'a>,
  ) -> RefAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: &crate::runtime::Stack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> RefAgalValue {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: &crate::runtime::Stack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> RefAgalValue {
    todo!()
  }
}
