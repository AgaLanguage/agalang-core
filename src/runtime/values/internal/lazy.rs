use parser::ast::BNode;

use crate::runtime::{
    env::RefEnvironment, get_instance_property_error, AgalInternal, AgalString, AgalThrow,
    AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

#[derive(Clone)]
pub struct AgalLazy(BNode);

impl AgalValuable for AgalLazy {
    fn to_value(self) -> AgalValue {
        AgalInternal::Lazy(self).to_value()
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<valor vago>".to_string()))
    }

    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        get_instance_property_error(stack, env, key, self.clone().to_value())
    }
}

impl PartialEq for AgalLazy {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
