use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::{
  functions_names, libraries, parser,
  runtime::{
    self, stack,
    values::{
      error_message, internal,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
    Stack,
  },
  util,
};

use super::{string::AgalString, AgalBoolean, AgalPrimitive};

type Integer = i32;
type Decimal = f32;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub enum AgalNumber {
  Integer(Integer),
  Decimal(Decimal),
  #[default]
  NaN,
  Infinity,
  NegInfinity,
}

macro_rules! a {
    ($type_val:ty, $type_other:ty, $trait_name:ident, $fn_name:ident, $token:tt) => {
      impl $trait_name<$type_other> for $type_val {
          type Output = AgalNumber;
          fn $fn_name(self, other: $type_other) -> AgalNumber {
            match (self, other) {
              (AgalNumber::Decimal(a), AgalNumber::Decimal(b)) => AgalNumber::Decimal(a $token b),
              (AgalNumber::Integer(a), AgalNumber::Integer(b)) => AgalNumber::Integer(a $token b),
              (a, b) => a.to_agal_dec() $token b.to_agal_dec(),
            }
          }
      }
    };
}
/// Genera el codigo nesesario para las implementaciones de las traits de operaciones
macro_rules! impl_agal {
  ($trait_name:ident $token:tt $fn_name:ident) => {
    a!(AgalNumber, AgalNumber, $trait_name, $fn_name, $token);
    a!(&AgalNumber, AgalNumber, $trait_name, $fn_name, $token);
    a!(AgalNumber, &AgalNumber, $trait_name, $fn_name, $token);
    a!(&AgalNumber, &AgalNumber, $trait_name, $fn_name, $token);
  };
}

impl_agal![Add + add];
impl_agal![Sub - sub];
impl_agal![Mul * mul];
impl_agal![Div / div];
impl_agal![Rem % rem];

impl AgalNumber {
  pub fn to_float(&self) -> Decimal {
    match self {
      Self::Integer(i) => *i as Decimal,
      Self::Decimal(f) => *f,
      Self::NaN => Decimal::NAN,
      Self::Infinity => Decimal::INFINITY,
      Self::NegInfinity => Decimal::NEG_INFINITY,
    }
  }
  pub fn floor(&self) -> Self {
    match self {
      Self::Decimal(f) => Self::Integer(f.floor() as Integer),
      Self::NaN | Self::Infinity | Self::NegInfinity | Self::Integer(_) => *self,
    }
  }
  pub fn to_usize(&self, stack: stack::RefStack) -> Result<usize, internal::AgalThrow> {
    match self {
      Self::Integer(i) => Ok(*i as usize),
      Self::Decimal(f) => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: "Los decimales no pueden ser tratados como enteros".to_string(),
        stack,
      }),
      Self::NaN => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: "No se puede convertir un NeN a entero".to_string(),
        stack,
      }),
      Self::Infinity => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: "No se puede convertir un número infinito a entero".to_string(),
        stack,
      }),
      Self::NegInfinity => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: "No se puede convertir un número negativo infinito a entero".to_string(),
        stack,
      }),
    }
  }
  pub fn to_agal_int(&self) -> Self {
    match self {
      Self::Integer(i) => *self,
      Self::Decimal(f) => Self::Integer(*f as Integer),
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegInfinity => Self::NegInfinity,
    }
  }
  pub fn to_agal_dec(&self) -> Self {
    match self {
      Self::Integer(i) => Self::Decimal(*i as Decimal),
      Self::Decimal(f) => *self,
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegInfinity => Self::NegInfinity,
    }
  }
  pub fn is_zero(&self) -> bool {
    match self {
      Self::Integer(0) | Self::Decimal(0.0) | Self::NaN => true,
      _ => false,
    }
  }
  pub fn neg(&self) -> Self {
    match self {
      Self::Integer(i) => Self::Integer(-i),
      Self::Decimal(f) => Self::Decimal(-f),
      Self::NaN => Self::NaN,
      Self::Infinity => Self::NegInfinity,
      Self::NegInfinity => Self::Infinity,
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
      Self::Integer(_) => "<Número entero>".to_string(),
      Self::Decimal(_) => "<Número decimal>".to_string(),
      Self::NaN => "<No es Número>".to_string(),
      Self::Infinity => "<Infinito>".to_string(),
      AgalNumber::NegInfinity => "<Infinito Negativo>".to_string(),
    }
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::string::AgalString, internal::AgalThrow> {
    Ok(super::string::AgalString::from_string(self.to_string()))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::YELLOW),
    )
  }
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(!self.is_zero()))
  }
  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: runtime::values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let x = right.clone();
    let x = x.borrow();
    let prim = if let AgalValue::Primitive(p) = &*x {
      &*p.borrow()
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }
      .to_result();
    };
    match (prim, operator) {
      (AgalPrimitive::Number(number), parser::NodeOperator::LessThan) => {
        AgalBoolean::new(self.less_than(number)).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::LessThanOrEqual) => {
        AgalBoolean::new(self.less_than(number) || self.equals(number)).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::GreaterThan) => {
        AgalBoolean::new(number.less_than(self)).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::GreaterThanOrEqual) => {
        AgalBoolean::new(self.equals(number) || number.less_than(self)).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::Equal) => {
        AgalBoolean::new(self.equals(number)).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::NotEqual) => {
        AgalBoolean::new(self.equals(number)).not().to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::And) => {
        if self.is_zero() {
          self.to_result()
        } else {
          right.to_result()
        }
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::Or) => {
        if self.is_zero() {
          right.to_result()
        } else {
          self.to_result()
        }
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::Plus) => (self + number).to_result(),
      (AgalPrimitive::Number(number), parser::NodeOperator::Minus) => (self - number).to_result(),
      (AgalPrimitive::Number(number), parser::NodeOperator::Multiply) => {
        (self * number).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::Division) => {
        (self / number).to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::FloorDivision) => {
        (self / number).floor().to_result()
      }
      (AgalPrimitive::Number(number), parser::NodeOperator::Modulo) => (self % number).to_result(),
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }
      .to_result(),
    }
  }
  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      functions_names::TO_AGAL_STRING => modules
        .get_module(":proto/Numero")
        .ok_or_else(|| internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
          stack: stack.clone(),
        })?
        .get_instance_property(stack, key, modules),
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }
  fn call(
    &self,
    stack: runtime::RefStack,
    this: runtime::values::DefaultRefAgalValue,
    args: Vec<runtime::values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    if (args.len() != 1) {
      return internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
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
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TYPE_ERROR_NUMBER.to_owned(),
        stack,
      }
      .to_result();
    };
    let num = if let AgalPrimitive::Number(n) = prim {
      n
    } else {
      return internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TYPE_ERROR_NUMBER.to_owned(),
        stack,
      }
      .to_result();
    };
    self.mul(&num).to_result()
  }
  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<Self, internal::AgalThrow> {
    Ok(self.clone())
  }
  fn equals(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Decimal(d1), Self::Decimal(d2)) => d1 == d2,
      (Self::Integer(i1), Self::Integer(i2)) => i1 == i2,
      (Self::NaN, Self::NaN) => false,
      (Self::Infinity, Self::Infinity) => true,
      (Self::NegInfinity, Self::NegInfinity) => true,
      (a, b) => a.to_agal_dec().equals(&b.to_agal_dec()),
    }
  }

  fn less_than(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Decimal(d1), Self::Decimal(d2)) => d1 < d2,
      (Self::Integer(i1), Self::Integer(i2)) => i1 < i2,
      (Self::NaN, _) | (_, Self::NaN) => false,
      (_, Self::Infinity) => true,
      (Self::NegInfinity, _) => true,
      _ => false,
    }
  }
}
impl ToString for AgalNumber {
  fn to_string(&self) -> String {
    match self {
      Self::Integer(i) => i.to_string(),
      Self::Decimal(f) => {
        let string = f.to_string();
        let clean_string = string.trim_end_matches('0').trim_end_matches('.');
        if clean_string.contains('.') {
          clean_string.to_string()
        } else {
          format!("{}.0", clean_string)
        }
      }
      Self::NaN => "NeN".to_string(),
      Self::Infinity => "Infinito".to_string(),
      Self::NegInfinity => "-Infinito".to_string(),
    }
  }
}
impl From<i32> for AgalNumber {
  fn from(val: i32) -> Self {
    Self::Integer(val)
  }
}
