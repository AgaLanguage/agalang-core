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
impl AgalValuable for AgalFunction {
    fn to_value(self) -> AgalValue {
        AgalComplex::Function(self).to_value()
    }
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<Funcion>".to_string()))
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(
            "\x1b[36m<Funcion>\x1b[39m".to_string(),
        ))
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        get_instance_property_error(stack, env, key, self.to_value())
    }
    fn call(
        self,
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
            &self.body.to_node(),
            stack,
            new_env.as_ref(),
            &modules_manager.clone(),
        );
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
