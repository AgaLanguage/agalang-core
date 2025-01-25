use crate::runtime::{self,values::{internal, traits::{self, AgalValuable as _, ToAgalValue as _}, AgalValue}};

use super::{string::AgalString, AgalPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalByte(u8, bool);
impl AgalByte {
  pub fn new(value: u8) -> AgalByte {
    AgalByte(value, false)
  }
  pub fn to_u8(&self) -> u8 {
    self.0
  }
}
impl traits::ToAgalValue for AgalByte {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Byte(self).to_value()
  }
}
impl traits::AgalValuable for AgalByte {
  fn to_agal_string(
    &self,
  ) -> Result<super::string::AgalString, internal::AgalThrow> {
    Ok(super::string::AgalString::from_string(format!(
      "0by{:08b}",
      self.0
    )))
  }
  fn to_agal_byte(&self, stack: parser::util::RefValue<runtime::Stack>) -> Result<AgalByte, internal::AgalThrow> {
    Ok(*self)
  }
  fn to_agal_console(&self, stack: parser::util::RefValue<runtime::Stack>, env: runtime::RefEnvironment) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string()?.add_prev(&format!("\x1b[33m")).add_post(&format!("\x1b[0m")))
  }
  fn to_agal_boolean(&self, stack: parser::util::RefValue<runtime::Stack>) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(self.0 != 0))
  }
}
