use std::rc::Rc;

use crate::{
  runtime::{
    env::RefEnvironment, get_instance_property_error, AgalInternal, AgalString, AgalThrow,
    AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
  },
  Modules,
};
#[derive(Clone)]
pub struct AgalNativeFunction<'a> {
  pub name: String,
  pub func: Rc<
    dyn Fn(
      Vec<RefAgalValue<'a>>,
      &Stack,
      RefEnvironment<'a>,
      &Modules<'a>,
      RefAgalValue<'a>,
    ) -> RefAgalValue<'a>,
  >,
}
impl<'a> AgalValuable<'a> for AgalNativeFunction<'a> {
  fn to_value(self) -> AgalValue<'a> {
    AgalInternal::NativeFunction(self.clone()).to_value()
  }
  fn to_agal_string(&self, _: &Stack, _: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
    Ok(AgalString::from_string(
      format!("<Funcion nativa {}>", self.name).as_str(),
    ))
  }
  fn to_agal_console(&self, _: &Stack, _: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
    Ok(AgalString::from_string(
      format!("\x1b[36m<Funcion nativa {}>\x1b[39m", self.name).as_str(),
    ))
  }
  fn get_instance_property(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    key: String,
  ) -> RefAgalValue {
    let value = self.clone().to_value();
    let a = get_instance_property_error(stack, env, key, &value);
    a
  }
  async fn call(
    &'a self,
    stack: &Stack,
    env: RefEnvironment<'a>,
    this: RefAgalValue<'a>,
    arguments: Vec<RefAgalValue<'a>>,
    modules_manager: &Modules<'a>,
  ) -> RefAgalValue<'a> {
    let v = (self.func)(arguments, stack, env, modules_manager, this);
    v
  }

  fn binary_operation(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    operator: &str,
    other: RefAgalValue<'a>,
  ) -> RefAgalValue {
    todo!()
  }

  fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    operator: &str,
  ) -> RefAgalValue {
    todo!()
  }
}
