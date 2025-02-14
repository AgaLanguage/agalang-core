use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

use error_message::TO_AGAL_CONSOLE;
use internal::AgalThrow;
use parser::{
  internal::ErrorNames,
  util::{self, RefValue},
};

mod error_message;

pub mod complex;
pub mod internal;
pub mod primitive;
pub mod traits;
use primitive::AgalBoolean;
use traits::{AgalValuable as _, ToAgalValue};

pub struct RefAgalValue<T: traits::AgalValuable + traits::ToAgalValue>(Rc<RefCell<T>>);
impl<T: traits::AgalValuable + traits::ToAgalValue> RefAgalValue<T> {
  pub fn new(value: T) -> Self {
    Self(Rc::new(RefCell::new(value)))
  }
  pub fn borrow(&self) -> std::cell::Ref<T> {
    self.0.as_ref().borrow()
  }
  pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
    self.0.borrow_mut()
  }
  pub fn replace(&self, value: T) {
    *self.borrow_mut() = value;
  }
  pub fn ptr(&self) -> *const T {
    let b = &*self.borrow();
    b as *const T
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
    self.borrow().clone()
  }
}
impl RefAgalValue<AgalValue> {
  pub fn is_return(&self) -> bool {
    self.un_ref().is_return()
  }
  pub fn is_break(&self) -> bool {
    self.un_ref().is_break()
  }
  pub fn is_continue(&self) -> bool {
    self.un_ref().is_continue()
  }
  pub fn is_stop(&self) -> bool {
    self.un_ref().is_stop()
  }
  pub fn is_never(&self) -> bool {
    self.un_ref().is_never()
  }
  pub fn to_result(&self) -> Result<RefAgalValue<AgalValue>, internal::AgalThrow> {
    self.un_ref().to_result()
  }
  pub fn into_return(&self) -> Option<DefaultRefAgalValue> {
    self.un_ref().into_return()
  }
}

impl<T: traits::AgalValuable + traits::ToAgalValue> Clone for RefAgalValue<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl<T: traits::AgalValuable + traits::ToAgalValue> traits::AgalValuable for RefAgalValue<T> {
  fn get_name(&self) -> String {
    self.borrow().get_name().to_string()
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.borrow().to_agal_string()
  }
  fn to_agal_byte(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    self.borrow().to_agal_byte(stack)
  }
  fn to_agal_boolean(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    self.borrow().to_agal_boolean(stack)
  }
  fn to_agal_console(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.borrow().to_agal_console(stack, env)
  }
  fn to_agal_value(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.borrow().to_agal_value(stack, env)
  }
  fn get_instance_property(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::RefEnvironment,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.borrow().get_instance_property(stack, env, key)
  }
  fn get_object_property(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::RefEnvironment,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.borrow().get_object_property(stack, env, key)
  }
  fn set_object_property(
    &mut self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self
      .borrow_mut()
      .set_object_property(stack, env, key, value)
  }
  async fn call(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    self.borrow().call(stack, env, this, args, modules).await
  }

  fn binary_operation(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
    right: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    self.borrow().binary_operation(stack, env, operator, right)
  }
  fn unary_operator(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
  ) -> ResultAgalValue {
    self.borrow().unary_operator(stack, env, operator)
  }
  fn unary_back_operator(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
  ) -> ResultAgalValue {
    self.borrow().unary_back_operator(stack, env, operator)
  }
  fn to_agal_array(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    self.borrow().to_agal_array(stack)
  }
  fn get_keys(&self) -> Vec<String> {
    self.borrow().get_keys()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    self.borrow().to_agal_number(stack)
  }

  fn equals(&self, other: &Self) -> bool {
    self.borrow().equals(&*other.borrow())
  }

  fn less_than(&self, other: &Self) -> bool {
    self.borrow().less_than(&*other.borrow())
  }
}

impl<T: traits::AgalValuable + traits::ToAgalValue> ToString for RefAgalValue<T> {
  fn to_string(&self) -> String {
    match self.try_to_string() {
      Ok(s) => s,
      Err(e) => e.to_string(),
    }
  }
}

#[derive(Clone)]
pub enum AgalValue {
  Complex(RefAgalValue<complex::AgalComplex>),
  Primitive(RefAgalValue<primitive::AgalPrimitive>),
  Internal(RefAgalValue<internal::AgalInternal>),
  Export(String, DefaultRefAgalValue),
  Continue,
  Never,
  Break,
  Null,
}
impl AgalValue {
  pub fn is_return(&self) -> bool {
    match self {
      AgalValue::Internal(i) => i.un_ref().is_return(),
      _ => false,
    }
  }
  pub fn is_never(&self) -> bool {
    match self {
      AgalValue::Never => true,
      _ => false,
    }
  }
  pub fn is_break(&self) -> bool {
    match self {
      AgalValue::Break => true,
      _ => false,
    }
  }
  pub fn is_continue(&self) -> bool {
    match self {
      AgalValue::Continue => true,
      _ => false,
    }
  }
  pub fn is_stop(&self) -> bool {
    self.is_return() || self.is_break() || self.is_continue()
  }
  pub fn to_result(&self) -> ResultAgalValue {
    match self {
      AgalValue::Internal(i) => i.un_ref().to_result(),
      AgalValue::Continue | AgalValue::Break | AgalValue::Never => Ok(AgalValue::Never.as_ref()),
      _ => Ok(self.clone().as_ref()),
    }
  }
  pub fn into_return(&self) -> Option<DefaultRefAgalValue> {
    match self {
      AgalValue::Internal(i) => i.un_ref().into_return(),
      _ => None,
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
      Self::Export(_, v) => v.get_name(),
      Self::Continue => "<Palabra clave Continuar>".to_string(),
      Self::Break => "<Palabra clave Romper>".to_string(),
    }
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_string(),
      Self::Primitive(p) => p.to_agal_string(),
      Self::Internal(i) => i.to_agal_string(),
      Self::Never | Self::Null => Ok(primitive::AgalString::from_string(
        super::NULL_KEYWORD.to_string(),
      )),
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_string(),
        stack: super::Stack::get_default().to_ref(),
      }),
    }
  }
  fn to_agal_number(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_number(stack),
      Self::Primitive(p) => p.to_agal_number(stack),
      Self::Internal(i) => i.to_agal_number(stack),
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TO_AGAL_NUMBER.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_byte(
    &self,
    stack: util::RefValue<super::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_byte(stack),
      Self::Primitive(p) => p.to_agal_byte(stack),
      Self::Internal(i) => i.to_agal_byte(stack),
      Self::Export(_, v) => v.to_agal_byte(stack),
      Self::Never | Self::Null | Self::Break | Self::Continue => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TO_AGAL_BYTE.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: util::RefValue<super::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_boolean(stack),
      Self::Primitive(p) => p.to_agal_boolean(stack),
      Self::Internal(i) => i.to_agal_boolean(stack),
      Self::Export(_, v) => v.to_agal_boolean(stack),
      Self::Never | Self::Null | Self::Break | Self::Continue => Ok(primitive::AgalBoolean::False),
    }
  }
  fn to_agal_console(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_console(stack, env),
      Self::Primitive(p) => p.to_agal_console(stack, env),
      Self::Internal(i) => i.to_agal_console(stack, env),
      Self::Export(_, v) => v.to_agal_console(stack, env),
      Self::Null => Ok(primitive::AgalString::from_string(
        crate::colors::Color::BRIGHT_WHITE.apply(super::NULL_KEYWORD),
      )),
      Self::Never => Ok(primitive::AgalString::from_string(
        crate::colors::Color::GRAY.apply(super::NOTHING_KEYWORD),
      )),
      Self::Break | Self::Continue => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: TO_AGAL_CONSOLE.to_string(),
        stack,
      }),
    }
  }
  fn to_agal_value(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_value(stack, env),
      Self::Primitive(p) => p.to_agal_value(stack, env),
      Self::Internal(i) => i.to_agal_value(stack, env),
      Self::Export(_, v) => v.to_agal_value(stack, env),
      Self::Never | Self::Null => Ok(primitive::AgalString::from_string(
        super::NULL_KEYWORD.to_string(),
      )),
      Self::Break | Self::Continue => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: TO_AGAL_CONSOLE.to_string(),
        stack,
      }),
    }
  }
  fn get_instance_property(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.get_instance_property(stack, env, key),
      Self::Primitive(p) => p.get_instance_property(stack, env, key),
      Self::Internal(i) => i.get_instance_property(stack, env, key),
      Self::Export(_, v) => v.get_instance_property(stack, env, key),
      Self::Never | Self::Null | Self::Break | Self::Continue => internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: format!(
          "No se puede obtener la propiedad '{}' de {}",
          key,
          self.to_agal_string()?.to_string()
        ),
        stack,
      }
      .to_result(),
    }
  }
  fn call(
    &self,
    stack: util::RefValue<super::Stack>,
    env: super::env::RefEnvironment,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: util::RefValue<crate::Modules>,
  ) -> Pin<Box<dyn Future<Output = Result<DefaultRefAgalValue, internal::AgalThrow>> + '_>> {
    Box::pin(async move {
      match self {
        Self::Complex(c) => c.call(stack, env, this, args, modules).await,
        Self::Primitive(p) => p.call(stack, env, this, args, modules).await,
        Self::Internal(i) => i.call(stack, env, this, args, modules).await,
        Self::Export(_, v) => v.call(stack, env, this, args, modules).await,
        Self::Never | Self::Null | Self::Break | Self::Continue => internal::AgalThrow::Params {
          type_error: parser::internal::ErrorNames::TypeError,
          message: error_message::CALL.to_string(),
          stack,
        }
        .to_result(),
      }
    })
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Complex(c) => self.get_keys(),
      Self::Internal(i) => self.get_keys(),
      Self::Primitive(p) => self.get_keys(),
      Self::Export(_, v) => v.get_keys(),
      Self::Never | Self::Null | Self::Break | Self::Continue => vec![],
    }
  }

  fn to_agal_array(
    &self,
    stack: RefValue<super::Stack>,
  ) -> Result<RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    match self {
      Self::Complex(c) => c.to_agal_array(stack),
      Self::Primitive(p) => p.to_agal_array(stack),
      Self::Internal(i) => i.to_agal_array(stack),
      Self::Export(_, v) => v.to_agal_array(stack),
      Self::Never | Self::Null | Self::Break | Self::Continue => Err(internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TO_AGAL_ARRAY.to_string(),
        stack,
      }),
    }
  }

  fn binary_operation(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
    right: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    match (self, operator) {
      (Self::Complex(c), _) => c.borrow().binary_operation(stack, env, operator, right),
      (Self::Primitive(p), _) => p.borrow().binary_operation(stack, env, operator, right),
      (Self::Internal(i), _) => i.borrow().binary_operation(stack, env, operator, right),
      (Self::Null, "==") => match right.un_ref() {
        Self::Null => AgalBoolean::True.to_result(),
        _ => Ok(AgalBoolean::False.to_ref_value()),
      },
      (Self::Null, "!=") => match right.un_ref() {
        Self::Null => AgalBoolean::False.to_result(),
        _ => Ok(AgalBoolean::True.to_ref_value()),
      },
      (Self::Never, "==") => match right.un_ref() {
        Self::Never => AgalBoolean::True.to_result(),
        _ => Ok(AgalBoolean::False.to_ref_value()),
      },
      (Self::Never, "!=") => match right.un_ref() {
        Self::Never => AgalBoolean::False.to_result(),
        _ => Ok(AgalBoolean::True.to_ref_value()),
      },
      (Self::Never | Self::Null, "||" | "??") => Ok(right),
      (Self::Never | Self::Null, "&&") => self.clone().to_result(),
      (_, _) => AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.clone().as_ref(), operator, right),
        stack,
      }
      .to_result(),
    }
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
  ) -> ResultAgalValue {
    match (self, operator) {
      (Self::Complex(c), _) => c.borrow().unary_back_operator(stack, env, operator),
      (Self::Primitive(p), _) => p.borrow().unary_back_operator(stack, env, operator),
      (Self::Internal(i), _) => i.borrow().unary_back_operator(stack, env, operator),
      (Self::Never | Self::Null, "?") => Self::Never.to_result(),
      (_, _) => AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: error_message::UNARY_BACK_OPERATOR.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn unary_operator(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    operator: &str,
  ) -> ResultAgalValue {
    match (self, operator) {
      (Self::Complex(c), _) => c.borrow().unary_operator(stack, env, operator),
      (Self::Primitive(p), _) => p.borrow().unary_operator(stack, env, operator),
      (Self::Internal(i), _) => i.borrow().unary_operator(stack, env, operator),
      (Self::Never | Self::Null, "?") => AgalBoolean::False.to_result(),
      (Self::Never | Self::Null, "!") => AgalBoolean::True.to_result(),
      (_, _) => AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: error_message::UNARY_OPERATOR.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn get_object_property(
    &self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    key: &str,
  ) -> ResultAgalValue {
    match self {
      Self::Complex(c) => c.borrow().get_object_property(stack, env, key),
      Self::Internal(i) => i.borrow().get_object_property(stack, env, key),
      Self::Primitive(p) => p.borrow().get_object_property(stack, env, key),
      _ => AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::GET_OBJECT_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<super::Stack>,
    env: super::RefEnvironment,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> ResultAgalValue {
    match self {
      Self::Complex(c) => c.borrow_mut().set_object_property(stack, env, key, value),
      Self::Internal(i) => i.borrow_mut().set_object_property(stack, env, key, value),
      Self::Primitive(p) => p.borrow_mut().set_object_property(stack, env, key, value),
      _ => AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
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
