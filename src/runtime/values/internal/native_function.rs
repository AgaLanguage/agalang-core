use std::rc::Rc;

use crate::{
    runtime::{AgalInternal, AgalThrow,
        env::RefEnvironment, get_instance_property_error, AgalString, AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack
    },
    Modules,
};

pub struct AgalNativeFunction {
    pub name: String,
    pub func: Rc<
        dyn Fn(Vec<RefAgalValue>, &Stack, RefEnvironment, &Modules, RefAgalValue) -> RefAgalValue,
    >,
}
impl Clone for AgalNativeFunction {
    fn clone(&self) -> Self {
        AgalNativeFunction {
            name: self.name.clone(),
            func: self.func.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}
impl AgalValuable for AgalNativeFunction {
    fn to_value(self) -> AgalValue {
        AgalInternal::NativeFunction(self).to_value()
    }
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "<Funcion nativa {}>",
            self.name
        )))
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[36m<Funcion nativa {}>\x1b[39m",
            self.name
        )))
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
        env: RefEnvironment,
        this: RefAgalValue,
        arguments: Vec<RefAgalValue>,
        modules_manager: &Modules
    ) -> RefAgalValue {
        let v = (self.func)(arguments, stack, env, modules_manager, this);
                v
    }
}
