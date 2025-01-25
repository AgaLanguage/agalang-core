use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
  runtime::{
    env::{RefEnvironment, FALSE_KEYWORD, NOTHING_KEYWORD, NULL_KEYWORD, TRUE_KEYWORD},
    Stack,
  },
  Modules,
};
use parser::{ast::Node, internal::ErrorNames};

pub mod primitive;
pub use primitive::*;
pub mod complex;
pub use complex::*;
pub mod internal;
pub use internal::*;
pub mod traits;
pub use traits::*;

pub enum AgalValue<'a> {
  Internal(AgalInternal<'a>),
  Primitive(AgalPrimitive<'a>),
  Complex(AgalComplex<'a>),
  Export(String, RefAgalValue<'a>),
  Return(RefAgalValue<'a>),
  Continue,
  Never,
  Break,
  Null,
}
impl<'a> PartialEq for AgalValue<'a> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Internal(a), Self::Internal(b)) => a == b,
      (Self::Primitive(a), Self::Primitive(b)) => a == b,
      (Self::Complex(a), Self::Complex(b)) => a == b,
      (Self::Export(name, value), Self::Export(on, ov)) => name == on && value == ov,
      (Self::Never, Self::Never) => true,
      (Self::Null, Self::Null) => true,
      (_, _) => false,
    }
  }
}
impl<'a> AgalValue<'a> {
  pub fn is_never(&self) -> bool {
    match self {
      Self::Never => true,
      _ => false,
    }
  }
  pub fn is_stop(&self) -> bool {
    match self {
      Self::Break | Self::Continue | Self::Return(_) | Self::Internal(AgalInternal::Throw(_)) => {
        true
      }
      _ => false,
    }
  }
  pub fn is_break(&self) -> bool {
    match self {
      Self::Break => true,
      _ => false,
    }
  }
  pub fn is_return(&self) -> bool {
    match self {
      Self::Return(_) => true,
      _ => false,
    }
  }
  pub fn is_throw(&self) -> bool {
    match self {
      Self::Internal(AgalInternal::Throw(_)) => true,
      _ => false,
    }
  }
  pub fn get_throw(&self) -> Option<AgalThrow> {
    match self {
      Self::Internal(AgalInternal::Throw(t)) => Some(t.clone()),
      _ => None,
    }
  }
  pub fn is_export(&self) -> bool {
    match self {
      Self::Export(_, _) => true,
      _ => false,
    }
  }
  pub fn get_export(&self) -> Option<(&String, &RefAgalValue<'a>)> {
    match self {
      Self::Export(name, value) => Some((name, value)),
      _ => None,
    }
  }
}
impl<'a> AgalValuableManager<'a> for AgalValue<'a> {
  fn to_value(self) -> Self {
    self
  }
  fn to_agal_number(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalNumber, AgalThrow> {
    match self {
      Self::Internal(i) => i.to_agal_number(stack, env),
      Self::Primitive(p) => p.to_agal_number(stack, env),
      Self::Complex(c) => c.to_agal_number(stack, env),
      Self::Null => Ok(AgalNumber::new(0f64)),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en numero".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_string(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    match self {
      Self::Null => Ok(AgalString::from_string(NULL_KEYWORD)),
      Self::Never => Ok(AgalString::from_string(NOTHING_KEYWORD)),
      Self::Internal(i) => i.to_agal_string(stack, env),
      Self::Complex(c) => c.to_agal_string(stack, env),
      Self::Primitive(p) => p.to_agal_string(stack, env),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en cadena".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalBoolean, AgalThrow> {
    match self {
      Self::Null => env
        .as_ref()
        .borrow()
        .get(stack, FALSE_KEYWORD, &Node::None)
        .as_ref()
        .borrow()
        .to_agal_boolean(stack, env.clone()),
      Self::Never => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en booleano".to_string(),
        stack: Box::new(stack.clone()),
      }),
      Self::Internal(i) => i.to_agal_boolean(stack, env),
      Self::Complex(c) => c.to_agal_boolean(stack, env),
      Self::Primitive(p) => p.to_agal_boolean(stack, env),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en booleano".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_array(&self, stack: &Stack) -> Result<&AgalArray<'a>, AgalThrow> {
    match self {
      Self::Internal(i) => i.to_agal_array(stack),
      Self::Complex(c) => c.to_agal_array(stack),
      Self::Primitive(p) => p.to_agal_array(stack),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Iterable"),
        message: "El valor no es iterable".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
    match self {
      Self::Internal(i) => i.to_agal_byte(stack),
      Self::Complex(c) => c.to_agal_byte(stack),
      Self::Primitive(p) => p.to_agal_byte(stack),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Iterable"),
        message: "El valor no es iterable".to_string(),
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_console(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    match self {
      Self::Internal(i) => i.to_agal_console(stack, env),
      Self::Complex(c) => c.to_agal_console(stack, env),
      Self::Primitive(p) => p.to_agal_console(stack, env),
      Self::Null => Ok(AgalString::from_string("\x1b[97mnulo\x1b[39m")),
      _ => Ok(AgalString::from_string("\x1b[90mnada\x1b[39m")),
    }
  }
  fn to_agal_value(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    match self {
      Self::Internal(i) => i.to_agal_value(stack, env),
      Self::Complex(c) => c.to_agal_value(stack, env),
      Self::Primitive(p) => p.to_agal_value(stack, env),
      _ => self.to_agal_console(stack, env),
    }
  }
  fn get_instance_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue {
    match self {
      Self::Internal(i) => i.get_instance_property(stack, env, key),
      Self::Complex(c) => c.get_instance_property(stack, env, key),
      Self::Primitive(p) => p.get_instance_property(stack, env, key),
      _ => AgalError::new(
        ErrorNames::CustomError("Error Propiedad"),
        format!("No se puede obtener la propiedad {}", key),
        Box::new(stack.clone()),
      )
      .to_ref_value(),
    }
  }
  fn get_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue<'a> {
    match self {
      Self::Internal(i) => i.get_object_property(stack, env, key),
      Self::Complex(c) => c.get_object_property(stack, env, key),
      Self::Primitive(p) => p.get_object_property(stack, env, key),
      _ => get_property_error(stack, env, key),
    }
  }
  fn set_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
    value: RefAgalValue,
  ) -> RefAgalValue {
    let data = self;
    match data {
      Self::Internal(i) => i.set_object_property(stack, env, key, value),
      Self::Complex(c) => c.set_object_property(stack, env, key, value),
      Self::Primitive(p) => p.set_object_property(stack, env, key, value),
      _ => Self::Never.as_ref(),
    }
  }
  fn binary_operation(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
    other: RefAgalValue<'a>,
  ) -> RefAgalValue {
    match self {
      Self::Internal(i) => i.binary_operation(stack, env, operator, other),
      Self::Complex(c) => c.binary_operation(stack, env, operator, other),
      Self::Primitive(p) => p.binary_operation(stack, env, operator, other),
      _ => {
        let a = self;
        let b = &*other.as_ref().borrow();
        AgalBoolean::new(a == b).to_ref_value()
      }
    }
  }
  fn unary_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue {
    match self {
      Self::Internal(i) => i.unary_operator(stack, env, operator),
      Self::Complex(c) => c.unary_operator(stack, env, operator),
      Self::Primitive(p) => p.unary_operator(stack, env, operator),
      _ => Self::Never.as_ref(),
    }
  }
  fn unary_back_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue {
    match self {
      Self::Internal(i) => i.unary_back_operator(stack, env, operator),
      Self::Complex(c) => c.unary_back_operator(stack, env, operator),
      Self::Primitive(p) => p.unary_back_operator(stack, env, operator),
      _ => Self::Never.as_ref(),
    }
  }
  async fn call(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    this: RefAgalValue<'a>,
    args: Vec<RefAgalValue<'a>>,
    modules_manager: &Modules<'a>,
  ) -> RefAgalValue<'a> {
    match self {
      Self::Internal(n) => n.call(stack, env, this, args, modules_manager).await,
      Self::Complex(c) => c.call(stack, env, this, args, modules_manager).await,
      Self::Primitive(p) => p.call(stack, env, this, args, modules_manager).await,
      _ => {
        let message = format!("No se puede llamar a {}", self.get_type());
        AgalThrow::Params {
          type_error: ErrorNames::CustomError("Error Llamada"),
          message,
          stack: Box::new(stack.clone()),
        }
        .to_ref_value()
      }
    }
  }

  fn get_type(&self) -> &'static str {
    match self {
      Self::Internal(i) => i.get_type(),
      Self::Complex(c) => c.get_type(),
      Self::Primitive(p) => p.get_type(),
      Self::Null => "Nulo",
      _ => "Nada",
    }
  }

  fn get_keys(&self) -> Vec<String> {
    match self {
      Self::Internal(i) => i.get_keys(),
      Self::Complex(c) => c.get_keys(),
      Self::Primitive(p) => p.get_keys(),
      _ => vec![],
    }
  }

  fn get_length(&self) -> usize {
    match self {
      Self::Internal(i) => i.get_length(),
      Self::Complex(c) => c.get_length(),
      Self::Primitive(p) => p.get_length(),
      _ => 0,
    }
  }

  fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) {
    match self {
      Self::Internal(i) => i.delete_object_property(stack, env, key),
      Self::Complex(c) => c.delete_object_property(stack, env, key),
      Self::Primitive(p) => p.delete_object_property(stack, env, key),
      _ => (),
    }
  }
}

#[derive(Clone)]
pub struct RefAgalValue<'a, V, M>
where
  V: AgalValuable<'a>,
  M: AgalValuableManager<'a>,
{
  value: Rc<RefCell<AgalValuableTrait<'a, V, M>>>,
}

impl<'a, V, M> RefAgalValue<'a, V, M>
where
  V: AgalValuable<'a>,
  M: AgalValuableManager<'a>,
{
  pub fn borrow(&self) -> std::cell::Ref<AgalValuableTrait<'a, V, M>> {
    self.value.as_ref().borrow()
  }
  pub fn borrow_mut(&self) -> std::cell::RefMut<AgalValuableTrait<'a, V, M>> {
    self.value.borrow_mut()
  }
}

impl<'a, V, M> std::ops::Deref for RefAgalValue<'a, V, M> {
  type Target = AgalValuableTrait<'a, V, M>;

  fn deref(&self) -> &Self::Target {
    let a = self.borrow();
  }
}
/*
#[derive(Clone)]

impl<T> From<Rc<RefCell<T>>> for RefAgalValue<T> {
  fn from(value: Rc<RefCell<T>>) -> Self {
    Self(value)
  }
}
impl<T> From<T> for RefAgalValue<T> {
  fn from(value: T) -> Self {
    Self(Rc::new(RefCell::new(value)))
  }
}

impl<T> Into<Rc<RefCell<T>>> for RefAgalValue<T> {
  fn into(self) -> Rc<RefCell<T>> {
    self.0
  }
}
*/

pub fn set_property_error<'a>(
  stack: &Stack,
  env: RefEnvironment,
  key: String,
  message: String,
) -> RefAgalValue<'a> {
  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Propiedad"),
    message: format!("No se puede obtener la propiedad {}: {}", key, message),
    stack: Box::new(stack.clone()),
  }
  .to_ref_value()
}
pub fn get_property_error<'a>(stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue<'a> {
  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Propiedad"),
    message: format!("No se puede obtener la propiedad {}", key),
    stack: Box::new(stack.clone()),
  }
  .to_ref_value()
}
pub fn delete_property_error<'a>(
  stack: &Stack,
  env: RefEnvironment<'a>,
  key: String,
) -> AgalValue<'a> {
  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Propiedad"),
    message: format!("No se puede eliminar la propiedad {}", key),
    stack: Box::new(stack.clone()),
  }
  .to_value()
}

pub fn get_instance_property_error<'a>(
  stack: &Stack,
  env: RefEnvironment<'a>,
  key: String,
  value: &AgalValue<'a>,
) -> RefAgalValue<'a> {
  let rc_stack: Rc<RefCell<Stack>> = Rc::new(RefCell::new(stack.clone()));
  match key.as_str() {
    "aCadena" => crate::runtime::AgalNativeFunction {
      name: "aCadena".to_string(),
      func: Rc::new({
        let e_stack = Rc::clone(&rc_stack);
        let e_env = Rc::clone(&env);
        move |_, _, _, _, _| -> RefAgalValue<'a> {
          let data = get_instance_property_value(e_stack.clone(), e_env.clone(), &key, value);
          data
        }
      }),
    }
    .to_ref_value(),
    "__aConsola__" => {
      let key_clone = key.clone();
      crate::runtime::AgalNativeFunction {
        name: "__aConsola__".to_string(),
        func: Rc::new({
          let e_stack = Rc::clone(&rc_stack);
          let e_env = Rc::clone(&env);
          move |_, _, _, _, _| -> RefAgalValue {
            let data =
              get_instance_property_value(e_stack.clone(), e_env.clone(), &key_clone, &value);
            data
          }
        }),
      }
      .to_ref_value()
    }
    "aNumero" => {
      let key_clone = key.clone();
      crate::runtime::AgalNativeFunction {
        name: "aNumero".to_string(),
        func: Rc::new({
          let e_stack = Rc::clone(&rc_stack);
          let e_env = Rc::clone(&env);
          move |_, _, _, _, _| -> RefAgalValue {
            let data =
              get_instance_property_value(e_stack.clone(), e_env.clone(), &key_clone, &value);
            data
          }
        }),
      }
      .to_ref_value()
    }
    "aBuleano" => {
      let key_clone = key.clone();
      crate::runtime::AgalNativeFunction {
        name: "aBuleano".to_string(),
        func: Rc::new({
          let e_stack = Rc::clone(&rc_stack);
          let e_env = Rc::clone(&env);
          move |_, _, _, _, _| -> RefAgalValue {
            let data =
              get_instance_property_value(e_stack.clone(), e_env.clone(), &key_clone, &value);
            data
          }
        }),
      }
      .to_ref_value()
    }
    "__aIterable__" => {
      let key_clone = key.clone();
      crate::runtime::AgalNativeFunction {
        name: "__aIterable__".to_string(),
        func: Rc::new({
          let e_stack = Rc::clone(&rc_stack);
          let e_env = Rc::clone(&env);
          move |_, _, _, _, _| -> RefAgalValue {
            let data =
              get_instance_property_value(e_stack.clone(), e_env.clone(), &key_clone, &value);
            data
          }
        }),
      }
      .to_ref_value()
    }
    _ => get_property_error(stack, env, key),
  }
}

fn get_instance_property_value<'a>(
  stack: Rc<RefCell<Stack>>,
  env: RefEnvironment<'a>,
  key: &str,
  value: &'a AgalValue<'a>,
) -> RefAgalValue<'a> {
  let stack = &stack.as_ref().borrow();
  match key {
    "aCadena" => {
      let data = value.to_agal_string(
        stack,
        env.as_ref().borrow().clone().crate_child(false).as_ref(),
      );
      match data {
        Ok(s) => s.to_ref_value(),
        Err(e) => e.to_ref_value(),
      }
    }
    "__aConsola__" => {
      let data = value.clone().to_agal_console(
        stack,
        env.as_ref().borrow().clone().crate_child(false).as_ref(),
      );
      match data {
        Ok(s) => s.to_ref_value(),
        Err(e) => e.to_ref_value(),
      }
    }
    "aNumero" => {
      let data = value.clone().to_agal_number(
        stack,
        env.as_ref().borrow().clone().crate_child(false).as_ref(),
      );
      match data {
        Ok(s) => s.to_ref_value(),
        Err(e) => e.to_ref_value(),
      }
    }
    "aBuleano" => {
      let data = value.to_agal_boolean(
        stack,
        env.as_ref().borrow().clone().crate_child(false).as_ref(),
      );
      match data {
        Ok(s) => s.to_ref_value(),
        Err(e) => e.to_ref_value(),
      }
    }
    "__aIterable__" => {
      let data = value.to_agal_array(stack);
      match data {
        Ok(s) => s.clone(),
        Err(e) => e.to_ref_value(),
      }
    }
    _ => get_property_error(stack, env, key.to_string()),
  }
}
pub fn binary_operation_error<'a>(
  stack: &Stack,
  operator: &str,
  left: RefAgalValue<'a>,
  right: RefAgalValue<'a>,
) -> RefAgalValue<'a> {
  let left = left.as_ref().borrow();
  let right = right.as_ref().borrow();

  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Operacion"),
    message: format!(
      "No se puede realizar la operacion {} {} {}",
      left.get_type(),
      operator,
      right.get_type()
    ),
    stack: Box::new(stack.clone()),
  }
  .to_ref_value()
}
pub fn unary_operation_error<'a>(
  stack: &Stack,
  operator: &str,
  value: RefAgalValue<'a>,
) -> RefAgalValue<'a> {
  let value = value.as_ref().borrow();

  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Operacion"),
    message: format!(
      "No se puede realizar la operacion {} {}",
      operator,
      value.get_type(),
    ),
    stack: Box::new(stack.clone()),
  }
  .to_ref_value()
}
pub fn unary_back_operation_error<'a>(
  stack: &Stack,
  operator: &str,
  value: RefAgalValue<'a>,
) -> RefAgalValue<'a> {
  let value = value.as_ref().borrow();

  if value.is_throw() && operator == "?" {
    return AgalValue::Null.as_ref();
  }

  AgalThrow::Params {
    type_error: ErrorNames::CustomError("Error Operacion"),
    message: format!(
      "No se puede realizar la operacion {} {}",
      value.get_type(),
      operator,
    ),
    stack: Box::new(stack.clone()),
  }
  .to_ref_value()
}
