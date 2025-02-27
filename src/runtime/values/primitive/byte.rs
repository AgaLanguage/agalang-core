use crate::{
  libraries, parser,
  runtime::{
    self,
    values::{
      error_message, internal,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  util,
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
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::string::AgalString, internal::AgalThrow> {
    Ok(super::string::AgalString::from_string(format!(
      "0by{:08b}",
      self.0
    )))
  }
  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalByte, internal::AgalThrow> {
    Ok(*self)
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
    Ok(super::boolean::AgalBoolean::new(self.0 != 0))
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::RefAgalValue<runtime::values::complex::AgalArray>, internal::AgalThrow>
  {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_ARRAY.to_owned(),
      stack,
    }
    .to_result()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: runtime::values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Primitive(p) = right.un_ref() {
      p.un_ref()
    } else {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_owned(),
        stack,
      });
    };
    match (operator, other) {
      (parser::NodeOperator::Equal, AgalPrimitive::Byte(other)) => {
        super::AgalBoolean::new(self.equals(&other)).to_result()
      }
      (parser::NodeOperator::NotEqual, AgalPrimitive::Byte(other)) => {
        super::AgalBoolean::new(!self.equals(&other)).to_result()
      }
      (
        parser::NodeOperator::BitMoveLeft,
        AgalPrimitive::Number(super::AgalNumber::Integer(int)),
      ) => super::AgalByte::new(self.0 << int).to_result(),
      (
        parser::NodeOperator::BitMoveLeft,
        AgalPrimitive::Number(super::AgalNumber::Integer(int)),
      ) => super::AgalByte::new(self.0 >> int).to_result(),
      (parser::NodeOperator::BitAnd, AgalPrimitive::Byte(other)) => {
        super::AgalByte::new(self.0 & other.0).to_result()
      }
      (parser::NodeOperator::BitOr, AgalPrimitive::Byte(other)) => {
        super::AgalByte::new(self.0 | other.0).to_result()
      }
      _ => Err(internal::AgalThrow::Params {
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
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    Ok(super::AgalNumber::from(self.0 as i32))
  }

  fn equals(&self, other: &Self) -> bool {
    self == other
  }

  fn less_than(&self, other: &Self) -> bool {
    self < other
  }
}
