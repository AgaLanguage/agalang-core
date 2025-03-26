use crate::{
  functions_names, libraries, parser, runtime::{
    self,
    values::{
      error_message,
      internal::{self, AgalThrow},
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
    FALSE_KEYWORD, TRUE_KEYWORD,
  }, util
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
  fn as_string(&self) -> String {
    format!("[{} {}]", self.get_name(), self.to_string())
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(super::AgalString::from(self))
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
  ) -> Result<Self, internal::AgalThrow> {
    Ok(*self)
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: runtime::values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let x = right.clone();
    let x = x.get();
    let prim = if let AgalValue::Primitive(p) = &*x {
      &*p.get()
    } else {
      return Err(AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      });
    };
    match (prim, operator) {
      (AgalPrimitive::Boolean(b), parser::NodeOperator::And) => self.and(b).to_result(),
      (AgalPrimitive::Boolean(b), parser::NodeOperator::Or) => self.or(b).to_result(),
      (AgalPrimitive::Boolean(b), parser::NodeOperator::Equal) => {
        AgalBoolean::new(self.equals(&b)).to_result()
      }
      (AgalPrimitive::Boolean(b), parser::NodeOperator::NotEqual) => {
        AgalBoolean::new(!self.equals(&b)).to_result()
      }
      _ => Err(AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }),
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
        .get_module(":proto/Buleano")
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

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    Ok(super::AgalNumber::from(self.as_bool() as i32))
  }

  fn equals(&self, other: &Self) -> bool {
    self == other
  }

  fn less_than(&self, other: &Self) -> bool {
    self.as_bool() < other.as_bool()
  }
}

impl std::fmt::Display for AgalBoolean {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      Self::False => FALSE_KEYWORD.to_string(),
      Self::True => TRUE_KEYWORD.to_string()
    })
  }
}