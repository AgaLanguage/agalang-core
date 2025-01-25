use crate::runtime::{
  self,
  values::{
    internal,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
};

use super::{string::AgalString, AgalPrimitive};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum AgalNumber {
  Integer(i32),
  Decimal(f32),
}

impl AgalNumber {
  pub fn to_agal_int(&self) -> Self {
    match self {
      AgalNumber::Integer(i) => *self,
      AgalNumber::Decimal(f) => AgalNumber::Integer(*f as i32),
    }
  }
  pub fn to_agal_dec(&self) -> Self {
    match self {
      AgalNumber::Integer(i) => AgalNumber::Decimal(*i as f32),
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
}
impl traits::ToAgalValue for AgalNumber {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Number(self).to_value()
  }
}
impl traits::AgalValuable for AgalNumber {
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
    Ok(
      self
        .to_agal_string()?
        .add_prev(&format!("\x1b[33m"))
        .add_post(&format!("\x1b[0m")),
    )
  }
  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(!self.is_zero()))
  }
}
