use parser::util::RefValue;

use crate::{
  colors,
  runtime::{
    self, stack,
    values::{
      error_message,
      internal::{self, AgalThrow},
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
    Stack,
  },
};

use super::{string::AgalString, AgalPrimitive};

type Integer = i32;
type Decimal = f32;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum AgalNumber {
  Integer(Integer),
  Decimal(Decimal),
}

impl AgalNumber {
  pub fn to_float(&self) -> f32 {
    match self {
      AgalNumber::Integer(i) => *i as f32,
      AgalNumber::Decimal(f) => *f,
    }
  }
  pub fn to_usize(&self, stack: RefValue<Stack>) -> Result<usize, internal::AgalThrow> {
    match self {
      AgalNumber::Integer(i) => Ok(*i as usize),
      AgalNumber::Decimal(f) => Err(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: "Los decimales no pueden ser tratados como enteros".to_string(),
        stack,
      }),
    }
  }
  pub fn to_agal_int(&self) -> Self {
    match self {
      AgalNumber::Integer(i) => *self,
      AgalNumber::Decimal(f) => AgalNumber::Integer(*f as Integer),
    }
  }
  pub fn to_agal_dec(&self) -> Self {
    match self {
      AgalNumber::Integer(i) => AgalNumber::Decimal(*i as Decimal),
      AgalNumber::Decimal(f) => *self,
    }
  }
  pub fn is_zero(&self) -> bool {
    match self {
      AgalNumber::Integer(0) => true,
      AgalNumber::Decimal(0.0) => true,
      _ => false,
    }
  }
  pub fn mul(&self, other: &Self) -> Self {
    match (self, other) {
      (AgalNumber::Integer(a), AgalNumber::Integer(b)) => AgalNumber::Integer(a * b),
      (AgalNumber::Decimal(a), AgalNumber::Decimal(b)) => AgalNumber::Decimal(a * b),
      _ => AgalNumber::Integer(0),
    }
  }
}
impl traits::ToAgalValue for AgalNumber {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Number(self).to_value()
  }
}
impl traits::AgalValuable for AgalNumber {
  fn get_name(&self) -> String {
    match self {
      AgalNumber::Integer(_) => "<Número entero>".to_string(),
      AgalNumber::Decimal(_) => "<Número decimal>".to_string(),
    }
  }
  fn to_agal_string(&self) -> Result<super::string::AgalString, internal::AgalThrow> {
    match self {
      AgalNumber::Integer(i) => Ok(super::string::AgalString::from_string(i.to_string())),
      AgalNumber::Decimal(f) => Ok(super::string::AgalString::from_string(f.to_string())),
    }
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string()?.set_color(colors::Color::YELLOW))
  }
  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(!self.is_zero()))
  }

  fn get_keys(&self) -> Vec<String> {
    vec![]
  }

  fn to_agal_byte(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<runtime::values::RefAgalValue<runtime::values::complex::AgalArray>, internal::AgalThrow>
  {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "".to_owned(),
      stack,
    }
    .to_result()
  }

  async fn call(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: runtime::values::DefaultRefAgalValue,
    args: Vec<runtime::values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    if (args.len() != 1) {
      return internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::ONLY_ONE_NUMBER_MULT.to_owned(),
        stack,
      }
      .to_result();
    }
    let arg = args.get(0).unwrap().un_ref();
    let prim = if let AgalValue::Primitive(p) = arg {
      p.un_ref()
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TYPE_ERROR_NUMBER.to_owned(),
        stack,
      }
      .to_result();
    };
    let num = if let AgalPrimitive::Number(n) = prim {
      n
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TYPE_ERROR_NUMBER.to_owned(),
        stack,
      }
      .to_result();
    };
    self.mul(&num).to_result()
  }

  fn as_ref(self) -> runtime::values::RefAgalValue<Self>
  where
    Self: Sized + traits::ToAgalValue,
  {
    runtime::values::RefAgalValue::new(self)
  }

  fn try_to_string(&self) -> Result<String, internal::AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_agal_string()?.to_string())
  }

  fn to_agal_value(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<super::AgalString, internal::AgalThrow> {
    self.to_agal_console(stack, env)
  }

  fn to_agal_number(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    Ok(self.clone())
  }
}
