use crate::{
  colors,
  runtime::{
    self,
    values::{
      internal,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
};

use super::{string::AgalString, AgalPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
  fn get_name(&self) -> String {
    "Byte".to_string()
  }
  fn to_agal_string(&self,stack: runtime::RefStack) -> Result<super::string::AgalString, internal::AgalThrow> {
    Ok(super::string::AgalString::from_string(format!(
      "0by{:08b}",
      self.0
    )))
  }
  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<AgalByte, internal::AgalThrow> {
    Ok(*self)
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
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(self.0 != 0))
  }

  fn get_keys(&self) -> Vec<String> {
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
    todo!()
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
      self < other
    }
}
