use parser::{
  ast::{Node, NodeBlock, NodeIdentifier},
  util::List,
};

use crate::{
  runtime::{
    env::{RefEnvironment, THIS_KEYWORD},
    get_instance_property_error, interpreter, AgalComplex, AgalString, AgalThrow, AgalValuable,
    AgalValuableManager, AgalValue, RefAgalValue, Stack,
  },
  Modules,
};

#[derive(Clone, PartialEq)]
pub struct AgalFunction {
  args: List<NodeIdentifier>,
  body: NodeBlock,
  env: RefEnvironment,
}

impl AgalFunction {
  pub fn new(args: List<NodeIdentifier>, body: NodeBlock, env: RefEnvironment) -> AgalFunction {
    AgalFunction { args, body, env }
  }
}
impl<'a> AgalValuable<'a> for AgalFunction {
  fn to_value(self) -> AgalValue<'a> {
    AgalComplex::Function(self).to_value()
  }
  fn to_agal_string(&self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string("<Funcion>"))
  }
  fn to_agal_console(&self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string("\x1b[36m<Funcion>\x1b[39m"))
  }
  fn get_instance_property(&self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    let value: AgalValue<'a> = self.clone().to_value();
    get_instance_property_error(stack, env, key, &value)
  }
  async fn call(
    &self,
    stack: &Stack,
    _: RefEnvironment,
    this: RefAgalValue,
    arguments: Vec<RefAgalValue>,
    modules_manager: &Modules,
  ) -> RefAgalValue {
    let mut new_env = self.env.as_ref().borrow().clone().crate_child(false);
    new_env.set(THIS_KEYWORD, this);
    for (i, arg) in self.args.iter().enumerate() {
      let value = if i < arguments.len() {
        arguments[i].clone()
      } else {
        AgalValue::Never.as_ref()
      };
      new_env.define(
        stack,
        &arg.name,
        value,
        true,
        &Node::Identifier(arg.clone()),
      );
    }
    let value = interpreter(
      self.body.to_node().to_box(),
      stack.clone().to_ref(),
      new_env.as_ref(),
      modules_manager.clone().to_ref(),
    )
    .await
    .await;
    if value.as_ref().borrow().is_throw() {
      return value;
    }
    let value: &AgalValue = &value.as_ref().borrow();
    if let AgalValue::Return(returned) = value {
      return returned.clone();
    }
    AgalValue::Never.as_ref()
  }
}
