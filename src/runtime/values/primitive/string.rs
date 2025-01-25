use super::{
  internal,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalPrimitive, AgalValue,
};
use crate::runtime;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgalChar(char);

impl AgalChar {
  pub fn new(value: char) -> Self {
    Self(value)
  }
  pub fn as_char(&self) -> char {
    self.0
  }
}

impl traits::ToAgalValue for AgalChar {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Char(self).to_value()
  }
}
impl traits::AgalValuable for AgalChar {
  fn to_agal_string(&self) -> Result<AgalString, internal::AgalThrow> {
    Ok(AgalString(vec![*self]))
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    let char = self.as_char();
    Ok(AgalString::from_string(format!(
      "\x1b[34m'{}'\x1b[0m",
      if char == '\'' {
        "\\'".to_string()
      } else {
        char.to_string()
      }
    )))
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgalString(Vec<AgalChar>);
impl AgalString {
  pub fn from_string(value: String) -> Self {
    Self(value.chars().map(|c| AgalChar::new(c)).collect())
  }
  pub fn to_string(&self) -> String {
    self.0.iter().map(|c| c.0).collect()
  }
  pub fn add_prev(&self, other: &str) -> Self {
    let mut new = vec![];
    new.extend(other.chars().map(|c| AgalChar::new(c)));
    new.extend(self.0.iter().map(|c| *c));
    Self(new)
  }
  pub fn add_post(&self, other: &str) -> Self {
    let mut new = vec![];
    new.extend(self.0.iter().map(|c| *c));
    new.extend(other.chars().map(|c| AgalChar::new(c)));
    Self(new)
  }
}
impl traits::ToAgalValue for AgalString {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::String(self).to_value()
  }
}
impl traits::AgalValuable for AgalString {
  fn try_to_string(&self) -> Result<String, internal::AgalThrow> {
    Ok(self.to_string())
  }
  fn to_agal_string(&self) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.clone())
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    self.to_agal_string()
  }
  fn to_agal_value(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    let string = self.try_to_string()?;
    let string = if string.contains("'") && string.contains("\"") {
      format!("'{}'", string.replace("\'", "\\\'"))
    } else if string.contains("'") {
      format!("\"{}\"", string)
    } else {
      format!("'{}'", string)
    };
    Ok(AgalString::from_string(format!(
      "\x1b[32m{}\x1b[0m",
      string
    )))
  }
  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(!self.0.is_empty()))
  }
}
