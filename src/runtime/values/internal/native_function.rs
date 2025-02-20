use std::rc::Rc;

use parser::util::RefValue;

use crate::{
  colors,
  runtime::{
    env::RefEnvironment,
    stack::RefStack,
    values::{
      self, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  Modules,
};

use super::AgalInternal;

#[derive(Clone)]
pub struct AgalNativeFunction {
  pub name: String,
  pub func: Rc<
    dyn Fn(
      Vec<values::DefaultRefAgalValue>,
      RefStack,
      RefValue<Modules>,
      values::DefaultRefAgalValue,
    ) -> Result<values::DefaultRefAgalValue, super::AgalThrow>,
  >,
}
impl std::fmt::Debug for AgalNativeFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "NativeFunction({})", self.name)
  }
}
impl traits::AgalValuable for AgalNativeFunction {
  fn get_name(&self) -> String {
    "<Función nativa>".to_string()
  }
  fn to_agal_string(&self, stack: RefStack) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[nativo fn {}]",
      self.name
    )))
  }
  fn to_agal_console(&self, stack: RefStack) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::CYAN))
  }
  async fn call(
    &mut self,
    stack: RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    (self.func)(args, stack, modules, this)
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: crate::runtime::RefStack,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: crate::runtime::RefStack,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    todo!()
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}
impl traits::ToAgalValue for AgalNativeFunction {
  fn to_value(self) -> AgalValue {
    AgalInternal::NativeFunction(self).to_value()
  }
}
