use crate::{
  colors,
  runtime::{
    self,
    values::{
      error_message, internal::{self, AgalThrow}, traits::{self, AgalValuable as _, ToAgalValue as _}, AgalValue
    },
    FALSE_KEYWORD, TRUE_KEYWORD,
  },
};

use super::{string::AgalString, AgalPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AgalBoolean {
  False,
  #[default]
  True,
}
impl AgalBoolean {
  pub fn new(value: bool) -> Self {
    if value {
      Self::True
    } else {
      Self::False
    }
  }
  pub fn as_bool(&self) -> bool {
    self == &Self::True
  }
  pub fn not(&self) -> Self {
    match self {
      Self::False => Self::True,
      Self::True => Self::False,
    }
  }
  pub fn and(&self, other: &Self) -> Self {
    match (self, other) {
      (Self::True, Self::True) => Self::True,
      (_, _) => Self::False,
    }
  }
  pub fn or(&self, other: &Self) -> Self {
    match (self, other) {
      (Self::False, Self::False) => Self::False,
      (_, _) => Self::True,
    }
  }
}
impl traits::ToAgalValue for AgalBoolean {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Boolean(self).to_value()
  }
}
impl traits::AgalValuable for AgalBoolean {
  fn get_name(&self) -> String {
    "Buleano".to_string()
  }
  fn to_agal_string(&self,stack: runtime::RefStack) -> Result<AgalString, internal::AgalThrow> {
    Ok(super::AgalString::from_string(match self {
      Self::False => FALSE_KEYWORD.to_string(),
      Self::True => TRUE_KEYWORD.to_string(),
    }))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::YELLOW))
  }
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<Self, internal::AgalThrow> {
    Ok(*self)
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<super::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<runtime::values::RefAgalValue<runtime::values::complex::AgalArray>, internal::AgalThrow>
  {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let x = right.clone();
    let x = x.borrow();
    let prim = if let AgalValue::Primitive(p) = &*x {
      &*p.borrow()
    } else {
      return Err(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.to_ref_value(), operator, right.clone()),
        stack,
      });
    };
    match (prim,operator) {
      (AgalPrimitive::Boolean(b), "&&") => self.and(b).to_result(),
      (AgalPrimitive::Boolean(b), "||") => self.or(b).to_result(),
      _ => Err(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.to_ref_value(), operator, right.clone()),
        stack,
      }),
    }
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    this: runtime::values::DefaultRefAgalValue,
    args: Vec<runtime::values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    self == other
  }

  fn less_than(&self, other: &Self) -> bool {
    self.as_bool() < other.as_bool()
  }
}
