use std::{
  future::Future,
  pin::Pin,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{parser, util};
use complex::{AgalPromise, AgalPromiseData};
use error_message::TO_AGAL_CONSOLE;
use futures_util::FutureExt;
use internal::AgalThrow;

mod error_message;

pub mod complex;
pub mod internal;
pub mod primitive;
pub mod traits;
use primitive::AgalBoolean;
use traits::{AgalValuable, ToAgalValue};

use crate::libraries;

use super::RefStack;
#[derive(Debug)]
pub struct RefAgalValue<T: traits::AgalValuable + traits::ToAgalValue>(Arc<RwLock<T>>);
impl<T: traits::AgalValuable + traits::ToAgalValue> RefAgalValue<T> {
  pub fn new(value: T) -> Self {
    Self(Arc::new(RwLock::new(value)))
  }
  pub fn as_arc(&self) -> Arc<RwLock<T>> {
    self.0.clone()
  }
  pub fn get(&self) -> RwLockReadGuard<'_, T> {
    self.0.read().unwrap()
  }
  pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
    self.0.write().unwrap()
  }
  pub fn ptr(&self) -> *const T {
    let b = &*self.get();
    b as *const T
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue> Clone for RefAgalValue<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue + Clone> traits::ToAgalValue
  for RefAgalValue<T>
{
  fn to_value(self) -> AgalValue {
    self.un_ref().to_value()
  }
  fn to_result(self) -> Result<DefaultRefAgalValue, internal::AgalThrow>
  where
    Self: Sized,
  {
    self.un_ref().to_result()
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue + Clone> RefAgalValue<T> {
  pub fn un_ref(&self) -> T {
    self.get().clone()
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue> traits::AgalValuable for RefAgalValue<T> {
  fn get_name(&self) -> String {
    self.get().get_name()
  }
  fn as_string(&self) -> String {
    self.get().as_string()
  }
  fn to_agal_string(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.get().to_agal_string(stack, modules)
  }
  fn to_agal_byte(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    self.get().to_agal_byte(stack, modules)
  }
  fn to_agal_boolean(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    self.get().to_agal_boolean(stack, modules)
  }
  fn to_agal_console(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.get().to_agal_console(stack, modules)
  }
  fn get_instance_property(
    &self,
    stack: super::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.get().get_instance_property(stack, key, modules)
  }
  fn get_object_property(
    &self,
    stack: super::RefStack,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.get().get_object_property(stack, key)
  }
  fn set_object_property(
    &mut self,
    stack: super::RefStack,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.get_mut().set_object_property(stack, key, value)
  }
  fn call(
    &self,
    stack: super::RefStack,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> ResultAgalValue {
    self.get().call(stack, this, args, modules)
  }

  fn binary_operation(
    &self,
    stack: super::RefStack,
    operator: parser::NodeOperator,
    right: DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.get().binary_operation(stack, operator, right, modules)
  }
  fn to_agal_array(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    self.get().to_agal_array(stack, modules)
  }
  fn get_keys(&self) -> Vec<String> {
    self.get().get_keys()
  }

  fn to_agal_number(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    self.get().to_agal_number(stack, modules)
  }

  fn equals(&self, other: &Self) -> bool {
    self.get().equals(&*other.get())
  }

  fn less_than(&self, other: &Self) -> bool {
    self.get().less_than(&*other.get())
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue + ToString> ToString for RefAgalValue<T> {
  fn to_string(&self) -> String {
    self.to_string()
  }
}
impl RefAgalValue<AgalValue> {
  pub fn is_return(&self) -> bool {
    self.get().is_return()
  }
  pub fn is_break(&self) -> bool {
    self.get().is_break()
  }
  pub fn is_continue(&self) -> bool {
    self.get().is_continue()
  }
  pub fn is_stop(&self) -> bool {
    self.get().is_stop()
  }
  pub fn is_never(&self) -> bool {
    self.get().is_never()
  }
  pub fn to_result(&self) -> Result<RefAgalValue<AgalValue>, internal::AgalThrow> {
    self.un_ref().to_result()
  }
  pub fn into_return(&self) -> DefaultRefAgalValue {
    self.get().into_return()
  }
}
impl Default for RefAgalValue<AgalValue> {
  fn default() -> Self {
    Self::new(AgalValue::default())
  }
}
impl RefAgalValue<AgalPromise> {
  pub fn replace(&self) -> AgalPromiseData {
    let mut guard_mut = self.get_mut();
    std::mem::replace(&mut guard_mut.data, AgalPromiseData::default())
  }
  pub fn get_value(&self) -> Option<ResultAgalValue> {
    let guard = self.get();

    if let AgalPromiseData::Resolved(r) = &guard.data {
      return Some(r.clone());
    }
    None
  }
}
unsafe impl<T: traits::AgalValuable + traits::ToAgalValue> Send for RefAgalValue<T> {}
unsafe impl<T: traits::AgalValuable + traits::ToAgalValue> Sync for RefAgalValue<T> {}

#[derive(Clone, Debug, Default)]
pub enum AgalValue {
  Complex(RefAgalValue<complex::AgalComplex>),
  Primitive(RefAgalValue<primitive::AgalPrimitive>),
  Internal(RefAgalValue<internal::AgalInternal>),
  Export(String, DefaultRefAgalValue),
  Return(DefaultRefAgalValue),
  Continue,
  #[default]
  Never,
  Break,
  Null,
  Console,
}
impl AgalValue {
  pub fn is_return(&self) -> bool {
    match self {
      Self::Return(_) => true,
      _ => false,
    }
  }
  pub fn is_never(&self) -> bool {
    match self {
      Self::Never => true,
      _ => false,
    }
  }
  pub fn is_break(&self) -> bool {
    match self {
      Self::Break => true,
      _ => false,
    }
  }
  pub fn is_continue(&self) -> bool {
    match self {
      Self::Continue => true,
      _ => false,
    }
  }
  pub fn is_stop(&self) -> bool {
    self.is_return() || self.is_break() || self.is_continue()
  }
  pub fn to_result(&self) -> ResultAgalValue {
    match self {
      Self::Internal(i) => i.un_ref().to_result(),
      Self::Continue | Self::Break | Self::Never => Ok(Self::Never.as_ref()),
      _ => Ok(self.clone().as_ref()),
    }
  }
  pub fn into_return(&self) -> DefaultRefAgalValue {
    match self {
      Self::Return(value) => value.clone(),
      Self::Continue | Self::Break | Self::Never => Self::Never.as_ref(),
      value => value.clone().as_ref(),
    }
  }
}

impl traits::ToAgalValue for AgalValue {
  fn to_value(self) -> AgalValue {
    self
  }
  fn to_result(self) -> Result<DefaultRefAgalValue, internal::AgalThrow>
  where
    Self: Sized,
  {
    match self {
      Self::Internal(i) => i.to_result(),
      _ => Ok(self.as_ref()),
    }
  }
}
impl traits::AgalValuable for AgalValue {
  fn get_name(&self) -> String {
    match self {
      Self::Complex(c) => c.get_name(),
      Self::Primitive(p) => p.get_name(),
      Self::Internal(i) => i.get_name(),
      Self::Never => "Ninguno".to_string(),
      Self::Null => "Nulo".to_string(),
      Self::Export(_, v) | Self::Return(v) => v.get_name(),
      Self::Continue => "<Palabra clave Continuar>".to_string(),
      Self::Break => "<Palabra clave Romper>".to_string(),
      Self::Console => "<Palabra clave Consola>".to_string(),
    }
  }
  fn as_string(&self) -> String {
    match self {
      Self::Complex(c) => c.as_string(),
      Self::Primitive(p) => p.as_string(),
      Self::Internal(i) => i.as_string(),
      Self::Never => "Ninguno".to_string(),
      Self::Null => "Nulo".to_string(),
      Self::Export(_, v) | Self::Return(v) => v.as_string(),
      Self::Continue => "<Palabra clave Continuar>".to_string(),
      Self::Break => "<Palabra clave Romper>".to_string(),
      Self::Console => "<Palabra clave Consola>".to_string(),
    }
  }
  fn to_agal_string(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_string(stack, modules),
      Self::Primitive(p) => p.to_agal_string(stack, modules),
      Self::Internal(i) => i.to_agal_string(stack, modules),
      Self::Never | Self::Null => Ok(primitive::AgalString::from_string(
        super::NULL_KEYWORD.to_string(),
      )),
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_number(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_number(stack, modules),
      Self::Primitive(p) => p.to_agal_number(stack, modules),
      Self::Internal(i) => i.to_agal_number(stack, modules),
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_NUMBER.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_byte(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_byte(stack, modules),
      Self::Primitive(p) => p.to_agal_byte(stack, modules),
      Self::Internal(i) => i.to_agal_byte(stack, modules),
      Self::Export(_, v) | Self::Return(v) => v.to_agal_byte(stack, modules),
      Self::Never | Self::Null => Ok(primitive::AgalByte::new(0)),
      Self::Break | Self::Console | Self::Continue => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_BYTE.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_boolean(stack, modules),
      Self::Primitive(p) => p.to_agal_boolean(stack, modules),
      Self::Internal(i) => i.to_agal_boolean(stack, modules),
      Self::Export(_, v) | Self::Return(v) => v.to_agal_boolean(stack, modules),
      Self::Never | Self::Null | Self::Break | Self::Continue | Self::Console => {
        Ok(primitive::AgalBoolean::False)
      }
    }
  }
  fn to_agal_console(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_console(stack, modules),
      Self::Primitive(p) => p.to_agal_console(stack, modules),
      Self::Internal(i) => i.to_agal_console(stack, modules),
      Self::Export(_, v) | Self::Return(v) => v.to_agal_console(stack, modules),
      Self::Null => Ok(primitive::AgalString::from_string(
        util::Color::BRIGHT_WHITE.apply(super::NULL_KEYWORD),
      )),
      Self::Never => Ok(primitive::AgalString::from_string(
        util::Color::GRAY.apply(super::NOTHING_KEYWORD),
      )),
      Self::Break | Self::Continue | Self::Console => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: TO_AGAL_CONSOLE.to_string(),
        stack,
      }),
    }
  }
  fn get_instance_property(
    &self,
    stack: super::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.get_instance_property(stack, key, modules),
      Self::Primitive(p) => p.get_instance_property(stack, key, modules),
      Self::Internal(i) => i.get_instance_property(stack, key, modules),
      Self::Export(_, v) | Self::Return(v) => v.get_instance_property(stack, key, modules),
      Self::Never | Self::Null | Self::Break | Self::Continue | Self::Console => {
        internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: format!(
            "No se puede obtener la propiedad '{}' de {}",
            key,
            self.to_agal_string(stack.clone(), modules)?.to_string()
          ),
          stack,
        }
        .to_result()
      }
    }
  }
  fn call(
    &self,
    stack: super::RefStack,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> ResultAgalValue {
    match self {
      Self::Complex(c) => c.call(stack, this, args, modules),
      Self::Primitive(p) => p.call(stack, this, args, modules),
      Self::Internal(i) => i.call(stack, this, args, modules),
      Self::Export(_, v) | Self::Return(v) => v.call(stack, this, args, modules),
      Self::Never | Self::Null | Self::Break | Self::Continue | Self::Console => {
        internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::CALL.to_string(),
          stack,
        }
        .to_result()
      }
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Complex(c) => self.get_keys(),
      Self::Internal(i) => self.get_keys(),
      Self::Primitive(p) => self.get_keys(),
      Self::Export(_, v) | Self::Return(v) => v.get_keys(),
      Self::Never | Self::Null | Self::Break | Self::Continue | Self::Console => vec![],
    }
  }

  fn to_agal_array(
    &self,
    stack: super::RefStack,
    modules: libraries::RefModules,
  ) -> Result<RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_array(stack, modules),
      Self::Primitive(p) => p.to_agal_array(stack, modules),
      Self::Internal(i) => i.to_agal_array(stack, modules),
      Self::Export(_, v) | Self::Return(v) => v.to_agal_array(stack, modules),
      Self::Never | Self::Null | Self::Break | Self::Continue | Self::Console => {
        Err(internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::TO_AGAL_ARRAY.to_string(),
          stack,
        })
      }
    }
  }

  fn binary_operation(
    &self,
    stack: super::RefStack,
    operator: parser::ast::NodeOperator,
    right: DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    match (self, operator, &*right.clone().get()) {
      (Self::Complex(c), _, _) => c.get().binary_operation(stack, operator, right, modules),
      (Self::Primitive(p), _, _) => p.get().binary_operation(stack, operator, right, modules),
      (Self::Internal(i), _, _) => i.get().binary_operation(stack, operator, right, modules),
      (Self::Null, parser::ast::NodeOperator::NotEqual, Self::Null)
      | (Self::Null, parser::ast::NodeOperator::Equal, _)
      | (Self::Never, parser::ast::NodeOperator::NotEqual, Self::Never)
      | (Self::Never, parser::ast::NodeOperator::Equal, _) => AgalBoolean::False.to_result(),
      (Self::Null, parser::ast::NodeOperator::Equal, Self::Null)
      | (Self::Null, parser::ast::NodeOperator::NotEqual, _)
      | (Self::Never, parser::ast::NodeOperator::Equal, Self::Never)
      | (Self::Never, parser::ast::NodeOperator::NotEqual, _) => AgalBoolean::True.to_result(),
      (
        Self::Never | Self::Null,
        parser::ast::NodeOperator::Or | parser::ast::NodeOperator::Nullish,
        _,
      ) => Ok(right),
      (Self::Never | Self::Null, parser::ast::NodeOperator::And, _) => self.clone().to_result(),
      (_, _, _) => AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }
      .to_result(),
    }
  }

  fn get_object_property(&self, stack: super::RefStack, key: &str) -> ResultAgalValue {
    match self {
      Self::Complex(c) => c.get().get_object_property(stack, key),
      Self::Internal(i) => i.get().get_object_property(stack, key),
      Self::Primitive(p) => p.get().get_object_property(stack, key),
      _ => AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_OBJECT_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn set_object_property(
    &mut self,
    stack: super::RefStack,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> ResultAgalValue {
    match self {
      Self::Complex(c) => c.get_mut().set_object_property(stack, key, value),
      Self::Internal(i) => i.get_mut().set_object_property(stack, key, value),
      Self::Primitive(p) => p.get_mut().set_object_property(stack, key, value),
      _ => AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::SET_OBJECT_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Complex(c1), Self::Complex(c2)) => c1.equals(c2),
      (Self::Internal(i1), Self::Internal(i2)) => i1.equals(i2),
      (Self::Primitive(p1), Self::Primitive(p2)) => p1.equals(p2),
      (Self::Never, Self::Never) => true,
      (Self::Null, Self::Null) => true,
      (_, _) => false,
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Complex(c1), Self::Complex(c2)) => c1.less_than(c2),
      (Self::Internal(i1), Self::Internal(i2)) => i1.less_than(i2),
      (Self::Primitive(p1), Self::Primitive(p2)) => p1.less_than(p2),
      (_, _) => false,
    }
  }
}
pub type DefaultRefAgalValue = RefAgalValue<AgalValue>;
pub type ResultAgalValue = Result<DefaultRefAgalValue, internal::AgalThrow>;
