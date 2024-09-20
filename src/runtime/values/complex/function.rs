use util::List;

use crate::{frontend::ast::{Node, NodeBlock, NodeIdentifier}, runtime::{env::{RefEnvironment, THIS_KEYWORD}, get_instance_property_error, interpreter, AgalValuable, AgalValue, RefAgalValue, Stack}};

#[derive(Clone, PartialEq)]
pub struct AgalFunction {
    args: List<NodeIdentifier>,
    body: NodeBlock,
    env: RefEnvironment,
}

impl AgalFunction {
    pub fn new(args: List<NodeIdentifier>, body: NodeBlock, env: RefEnvironment) -> AgalFunction {
      AgalFunction {args,body,env}
    }
}
impl AgalValuable for AgalFunction {
  fn to_value(self) -> crate::runtime::AgalValue {
      crate::runtime::AgalValue::Function(self)
  }
  fn get_instance_property(self, stack: &crate::runtime::Stack, env: RefEnvironment, key: String)
          -> crate::runtime::RefAgalValue {
      get_instance_property_error(stack, env, key, self.to_value())
  }
  fn call(
    self,
    stack: &Stack,
    _: RefEnvironment,
    this: RefAgalValue,
    arguments: Vec<RefAgalValue>) -> RefAgalValue{
    
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
    let value = interpreter(&self.body.to_node(), stack, self.env);
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