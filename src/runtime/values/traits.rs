use parser::util::RefValue;

use crate::{runtime, Modules};

use super::{
  complex::AgalArray, internal, primitive, AgalValue, DefaultRefAgalValue, RefAgalValue,
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
pub trait AgalValuable {
  fn get_name(&self) -> String;
  fn as_ref(self) -> RefAgalValue<Self>
  where
    Self: Sized + ToAgalValue,
  {
    RefAgalValue::new(self)
  }
  fn get_keys(&self) -> Vec<String>;
  fn try_to_string(&self, stack: runtime::RefStack) -> Result<String, internal::AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_agal_string(stack)?.to_string())
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalString, internal::AgalThrow>;
  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow>;
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow>;
  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow>;
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack)?
        .set_color(crate::colors::Color::MAGENTA),
    )
  }
  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<RefAgalValue<AgalArray>, internal::AgalThrow>;
  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow>;
  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow>;
  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow>;
  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow>;
  async fn call(
    &self,
    stack: runtime::RefStack,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow>;
  /// self == other
  fn equals(&self, other: &Self) -> bool;
  /// self < other
  fn less_than(&self, other: &Self) -> bool;
}
