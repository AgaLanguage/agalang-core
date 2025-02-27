use crate::{
  libraries,
  runtime::{
    self,
    values::{traits, DefaultRefAgalValue},
  },
};

#[derive(Debug, Clone)]
pub struct AgalImmutable(DefaultRefAgalValue);
impl AgalImmutable {
  pub fn new(value: DefaultRefAgalValue) -> Self {
    Self(value)
  }
}
impl AgalImmutable {
  pub fn get_value(&self) -> DefaultRefAgalValue {
    self.0.clone()
  }
}
impl traits::ToAgalValue for AgalImmutable {
  fn to_value(self) -> crate::runtime::values::AgalValue {
    crate::runtime::values::internal::AgalInternal::Immutable(self).to_value()
  }
}
impl traits::AgalValuable for AgalImmutable {
  fn get_name(&self) -> String {
    format!("Immutable({})", self.0.get_name())
  }

  fn get_keys(&self) -> Vec<String> {
    self.0.get_keys()
  }

  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::primitive::AgalString, super::AgalThrow> {
    self.0.to_agal_string(stack, modules)
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::primitive::AgalByte, super::AgalThrow> {
    self.0.to_agal_byte(stack, modules)
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::primitive::AgalBoolean, super::AgalThrow> {
    self.0.to_agal_boolean(stack, modules)
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::primitive::AgalNumber, super::AgalThrow> {
    self.0.to_agal_number(stack, modules)
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<
    crate::runtime::values::RefAgalValue<crate::runtime::values::complex::AgalArray>,
    super::AgalThrow,
  > {
    self.0.to_agal_array(stack, modules)
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: crate::parser::NodeOperator,
    right: DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, super::AgalThrow> {
    self.0.binary_operation(stack, operator, right, modules)
  }

  fn get_object_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<DefaultRefAgalValue, super::AgalThrow> {
    self.0.get_object_property(stack, key)
  }

  fn set_object_property(
    &mut self,
    stack: crate::runtime::RefStack,
    key: &str,
    value: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, super::AgalThrow> {
    super::AgalThrow::Params {
      type_error: crate::parser::ErrorNames::TypeError,
      message: "No se puede modificar un valor inmutable".into(),
      stack,
    }
    .to_result()
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<DefaultRefAgalValue, super::AgalThrow> {
    self.0.get_instance_property(stack, key, modules)
  }

  fn call(
    &self,
    stack: crate::runtime::RefStack,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, super::AgalThrow> {
    self.0.call(stack, this, args, modules)
  }

  fn equals(&self, other: &Self) -> bool {
    self.0.equals(&other.0)
  }

  fn less_than(&self, other: &Self) -> bool {
    self.0.less_than(&other.0)
  }
}
