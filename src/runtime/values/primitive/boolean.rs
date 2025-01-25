use crate::runtime::{
  self,
  values::{
    internal,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
  FALSE_KEYWORD, TRUE_KEYWORD,
};

use super::{string::AgalString, AgalPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgalBoolean {
  False,
  True,
}
impl AgalBoolean {
  pub fn new(value: bool) -> AgalBoolean {
    if value {
      AgalBoolean::True
    } else {
      AgalBoolean::False
    }
  }
  pub fn as_bool(&self) -> bool {
    self == &AgalBoolean::True
  }
}
impl traits::ToAgalValue for AgalBoolean {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Boolean(self).to_value()
  }
}
impl traits::AgalValuable for AgalBoolean {
  fn to_agal_string(&self) -> Result<AgalString, internal::AgalThrow> {
    Ok(super::string::AgalString::from_string(match self {
      AgalBoolean::False => FALSE_KEYWORD.to_string(),
      AgalBoolean::True => TRUE_KEYWORD.to_string(),
    }))
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
  ) -> Result<AgalBoolean, internal::AgalThrow> {
    Ok(*self)
  }
}
