use std::rc::Rc;

use parser::util::RefValue;

use crate::{
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
    Ok(
      self
        .to_agal_string()?
        .add_prev(&format!("\x1b[36m"))
        .add_post(&format!("\x1b[0m")),
    )
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
}
impl traits::ToAgalValue for AgalNativeFunction {
  fn to_value(self) -> AgalValue {
    AgalInternal::NativeFunction(self).to_value()
  }
}
