use std::rc::Rc;

use parser::util::RefValue;

use crate::{
  colors,
  runtime::{
    env::RefEnvironment,
    stack::Stack,
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
      RefValue<Stack>,
      RefEnvironment,
      RefValue<Modules>,
      values::DefaultRefAgalValue,
    ) -> Result<values::DefaultRefAgalValue, super::AgalThrow>,
  >,
}
impl traits::AgalValuable for AgalNativeFunction {
  fn get_name(&self) -> String {
    "<Función nativa>".to_string()
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[nativo fn {}]",
      self.name
    )))
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<Stack>,
    env: RefEnvironment,
  ) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(self.to_agal_string()?.set_color(colors::Color::CYAN))
  }
  async fn call(
    &self,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    (self.func)(args, stack, env, modules, this)
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<crate::runtime::Stack>,
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
