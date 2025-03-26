use crate::{libraries, parser, runtime, util};

use super::{
  complex, error_message, internal, primitive, AgalValue, DefaultRefAgalValue, RefAgalValue,
  ResultAgalValue,
};

pub trait ToAgalValue: AgalValuable {
  fn to_value(self) -> AgalValue;
  fn to_ref_value(self) -> DefaultRefAgalValue
  where
    Self: Sized,
  {
    self.to_value().as_ref()
  }
  fn to_result(self) -> Result<DefaultRefAgalValue, internal::AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_ref_value())
  }
}
pub trait AgalValuable: Send {
  fn get_name(&self) -> String;
  fn as_ref(self) -> RefAgalValue<Self>
  where
    Self: Sized + ToAgalValue,
  {
    RefAgalValue::new(self)
  }
  fn get_keys(&self) -> Vec<String> {
    vec![]
  }
  fn as_string(&self) -> String {
    format!("[ {} ]", self.get_name())
  }
  fn try_to_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<String, internal::AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_agal_string(stack, modules)?.to_string())
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(self.get_name()))
  }
  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_BYTE.into(),
      stack,
    }
    .to_result()
  }
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_BOOLEAN.into(),
      stack,
    }
    .to_result()
  }
  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.into(),
      stack,
    }
    .to_result()
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::MAGENTA),
    )
  }
  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_ARRAY.into(),
      stack,
    }
    .to_result()
  }
  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()).into(),
      stack,
    }
    .to_result()
  }
  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::GET_OBJECT_PROPERTY.into(),
      stack,
    }
    .to_result()
  }
  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::SET_OBJECT_PROPERTY.into(),
      stack,
    }
    .to_result()
  }
  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::GET_INSTANCE_PROPERTY.into(),
      stack,
    }
    .to_result()
  }
  fn call(
    &self,
    stack: runtime::RefStack,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::CALL.into(),
      stack,
    }
    .to_result()
  }
  /// self == other
  fn equals(&self, other: &Self) -> bool;
  /// self < other
  fn less_than(&self, other: &Self) -> bool;
}
