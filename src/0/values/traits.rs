use std::{cell::RefCell, rc::Rc};

use parser::{internal::ErrorNames, util::RefValue};

use crate::{
  runtime::{
    env::{RefEnvironment, TRUE_KEYWORD},
    Environment, Stack,
  },
  Modules,
};

use super::{
  binary_operation_error, delete_property_error, get_instance_property_error,
  get_instance_property_value, get_property_error, set_property_error, unary_back_operation_error,
  unary_operation_error, AgalArray, AgalBoolean, AgalByte, AgalNumber, AgalPrimitive, AgalString,
  AgalThrow, AgalValue, RefAgalValue,
};

pub trait AgalValuableManager<'a>
where
  Self: Sized + PartialEq,
{
  fn as_ref(self) -> RefValue<Self> {
    Rc::new(RefCell::new(self))
  }
  fn to_value(self) -> AgalValue<'a>;
  fn to_ref_value(self) -> RefAgalValue<'a> {
    self.to_value().as_ref()
  }
  fn get_type(&self) -> &'static str;
  fn get_keys(&self) -> Vec<String>;
  fn get_length(&self) -> usize;
  // types
  fn to_agal_number(&self, stack: &Stack, env: RefEnvironment<'a>)
    -> Result<AgalNumber, AgalThrow>;
  fn to_agal_string(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow>;
  fn to_agal_boolean(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalBoolean, AgalThrow>;
  fn to_agal_array(&self, stack: &Stack) -> Result<RefValue<AgalArray<'a>>, AgalThrow>;
  fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow>;

  // utils
  fn to_agal_value(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow>;
  fn to_agal_console(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow>;

  fn binary_operation(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
    other: RefAgalValue<'a>,
  ) -> RefAgalValue<'a>;

  fn unary_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue;

  fn unary_back_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue;
  // object methods
  fn get_object_property(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue<'a>;
  fn set_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
    value: RefAgalValue,
  ) -> RefAgalValue;
  fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String);
  // instance methods
  fn get_instance_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue;

  // values
  async fn call(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    this: RefAgalValue<'a>,
    args: Vec<RefAgalValue<'a>>,
    modules: &Modules<'a>,
  ) -> RefAgalValue<'a>;
}

pub trait AgalValuable<'a>
where
  Self: Sized,
{
  fn as_ref(self) -> RefValue<Self> {
    Rc::new(RefCell::new(self))
  }
  fn to_value(self) -> AgalValue<'a>;
  fn to_ref_value(self) -> RefAgalValue<'a> {
    self.to_value().as_ref()
  }
  fn get_keys(&'a self) -> Vec<String> {
    vec![]
  }
  fn get_length(&'a self) -> usize {
    0
  }
  // types
  fn to_agal_number(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::CustomError("Error Parseo"),
      message: "No se pudo convertir en numero",
      stack: Box::new(stack.clone()),
    })
  }
  fn to_agal_string(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    Ok(AgalString::from_string("<interno>"))
  }
  fn to_agal_boolean(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalBoolean, AgalThrow> {
    let value_rc = {
      let env = &env.as_ref().borrow();
      env.get(stack, TRUE_KEYWORD, stack.get_value())
    };

    let value_ref = value_rc.as_ref().borrow();

    let value: &AgalValue = &*value_ref;

    match value {
      &AgalValue::Primitive(AgalPrimitive::Boolean(b)) => Ok(b),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en booleano",
        stack: Box::new(stack.clone()),
      }),
    }
  }

  fn to_agal_array(&self, stack: &Stack) -> Result<RefValue<AgalArray<'a>>, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::CustomError("Error Iterable"),
      message: "El valor no es iterable",
      stack: Box::new(stack.clone()),
    })
  }
  fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: ErrorNames::TypeError,
      message: "El valor no es un byte",
      stack: Box::new(stack.clone()),
    })
  }

  // utils
  fn to_agal_value(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    self.to_agal_console(stack, env)
  }
  fn to_agal_console(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow>;

  fn binary_operation(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    operator: &str,
    other: RefAgalValue<'a>,
  ) -> RefAgalValue;

  fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue;

  fn unary_back_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str)
    -> RefAgalValue;
  // object methods
  fn get_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue<'a> {
    get_property_error(stack, env, key)
  }
  fn set_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment,
    key: String,
    value: RefAgalValue,
  ) -> RefAgalValue {
    set_property_error(stack, env, key, "No se puede asignar".to_string())
  }
  fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment, key: String) {
    delete_property_error(stack, env, key);
  }
  // instance methods
  fn get_instance_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue;

  // values
  async fn call(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    this: RefAgalValue<'a>,
    args: Vec<RefAgalValue<'a>>,
    modules: &Modules<'a>,
  ) -> RefAgalValue {
    AgalValue::Never.as_ref()
  }
}

pub enum AgalValuableTrait<'a, V, M>
where
  V: AgalValuable<'a>,
  M: AgalValuableManager<'a>,
{
  AgalValue(&'a V),
  AgalManager(&'a M),
  _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a, V, M> Clone for AgalValuableTrait<'a, V, M>
where
  V: AgalValuable<'a>,
  M: AgalValuableManager<'a>,
{
  fn clone(&self) -> AgalValuableTrait<'a, V, M> {
    match self {
      Self::AgalManager(m) => Self::AgalManager(*m),
      Self::AgalValue(v) => Self::AgalValue(*v),
      _ => todo!("invaluable"),
    }
  }
}

impl<'a, V, M> AgalValuableTrait<'a, V, M>
where
  V: AgalValuable<'a>,
  M: AgalValuableManager<'a>,
{
  fn get_keys(&'a self) -> Vec<String> {
    match self {
      Self::AgalValue(v) => v.get_keys(),
      Self::AgalManager(m) => m.get_keys(),
      _ => vec![],
    }
  }
  fn get_length(&'a self) -> usize {
    match self {
      Self::AgalValue(v) => v.get_length(),
      Self::AgalManager(m) => m.get_length(),
      _ => 0,
    }
  }
  // types
  fn to_agal_number(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalNumber, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_number(stack, env),
      Self::AgalManager(m) => m.to_agal_number(stack, env),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Parseo"),
        message: "No se pudo convertir en numero",
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
      Self::AgalValue(v) => v.to_agal_string(stack, env),
      Self::AgalManager(m) => m.to_agal_string(stack, env),
      _ => Ok(AgalString::from_string("<interno>")),
    }
  }
  fn to_agal_boolean(
    &self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalBoolean, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_boolean(stack, env),
      Self::AgalManager(m) => m.to_agal_boolean(stack, env),
      _ => {
        let value_rc = {
          let env = &env.as_ref().borrow();
          env.get(stack, TRUE_KEYWORD, stack.get_value())
        };

        let value_ref = value_rc.as_ref().borrow();

        let value = &*value_ref;

        match value {
          &AgalValue::Primitive(AgalPrimitive::Boolean(b)) => Ok(b),
          _ => Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo"),
            message: "No se pudo convertir en booleano",
            stack: Box::new(stack.clone()),
          }),
        }
      }
    }
  }

  fn to_agal_array(&self, stack: &Stack) -> Result<RefValue<AgalArray<'a>>, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_array(stack),
      Self::AgalManager(m) => m.to_agal_array(stack),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Iterable"),
        message: "El valor no es iterable",
        stack: Box::new(stack.clone()),
      }),
    }
  }
  fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_byte(stack),
      Self::AgalManager(m) => m.to_agal_byte(stack),
      _ => Err(AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: "El valor no es un byte",
        stack: Box::new(stack.clone()),
      }),
    }
  }

  // utils
  fn to_agal_value(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_value(stack, env),
      Self::AgalManager(m) => m.to_agal_value(stack, env),
      _ => self.to_agal_console(stack, env),
    }
  }
  fn to_agal_console(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
  ) -> Result<AgalString<'a>, AgalThrow> {
    match self {
      Self::AgalValue(v) => v.to_agal_console(stack, env),
      Self::AgalManager(m) => m.to_agal_console(stack, env),
      _ => self.to_agal_console(stack, env),
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
      Self::AgalValue(v) => v.binary_operation(stack, env, operator, other),
      Self::AgalManager(m) => m.binary_operation(stack, env, operator, other),
      _ => (AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: "Error al realizar la operacion",
        stack: Box::new(stack.clone()),
      })
      .to_ref_value(),
    }
  }

  fn unary_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue {
    match self {
      Self::AgalValue(v) => v.unary_operator(stack, env, operator),
      Self::AgalManager(m) => m.unary_operator(stack, env, operator),
      _ => (AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: "Error al realizar la operacion",
        stack: Box::new(stack.clone()),
      })
      .to_ref_value(),
    }
  }

  fn unary_back_operator(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    operator: &str,
  ) -> RefAgalValue {
    match self {
      Self::AgalValue(v) => v.unary_operator(stack, env, operator),
      Self::AgalManager(m) => m.unary_operator(stack, env, operator),
      _ => (AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: "Error al realizar la operacion",
        stack: Box::new(stack.clone()),
      })
      .to_ref_value(),
    }
  }
  // object methods
  fn get_object_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue<'a> {
    match self {
      Self::AgalValue(v) => v.get_object_property(stack, env, key),
      Self::AgalManager(m) => m.get_object_property(stack, env, key),
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
    match self {
      Self::AgalValue(v) => v.set_object_property(stack, env, key, value),
      Self::AgalManager(m) => m.set_object_property(stack, env, key, value),
      _ => set_property_error(stack, env, key, "No se puede asignar".to_string()),
    }
  }
  fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment, key: String) {
    delete_property_error(stack, env, key);
  }
  // instance methods
  fn get_instance_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue {
    match self {
      Self::AgalValue(v) => v.get_instance_property(stack, env, key),
      Self::AgalManager(m) => m.get_instance_property(stack, env, key),
      _ => get_instance_property_value(stack.clone().to_ref(), env, &key, &AgalValue::Never),
    }
  }

  // values
  async fn call(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    this: RefAgalValue<'a>,
    args: Vec<RefAgalValue<'a>>,
    modules: &Modules<'a>,
  ) -> RefAgalValue {
    match self {
      Self::AgalValue(v) => v.call(stack, env, this, args, modules).await,
      Self::AgalManager(m) => m.call(stack, env, this, args, modules).await,
      _ => (AgalThrow::Params {
        type_error: ErrorNames::TypeError,
        message: "Error ejecutar el valor",
        stack: Box::new(stack.clone()),
      })
      .to_ref_value(),
    }
  }
}
