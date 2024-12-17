use parser::{ast::BNode, util::RefValue};

use crate::runtime::{
    env::RefEnvironment, get_instance_property_error, AgalInternal, AgalString, AgalThrow,
    AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

#[derive(Clone)]
pub struct AgalLazy<'a>{
    node: RefValue<BNode>,
    value: Option<RefAgalValue<'a>>,
}

impl<'a> AgalValuable<'a> for AgalLazy<'a> {
    fn to_value(self) -> AgalValue<'a> {
        AgalInternal::Lazy(self.clone()).to_value()
    }
    fn to_agal_console(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<valor vago>"))
    }

    fn get_instance_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
    ) -> RefAgalValue<'a> {
        let value = self.clone().to_value();
        get_instance_property_error(stack, env, key, &value)
    }
    
    fn binary_operation(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue<'a>,
    ) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
    
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
    
    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
}

impl<'a> PartialEq for AgalLazy<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
