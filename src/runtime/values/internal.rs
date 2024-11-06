use std::{cell::RefCell, rc::Rc};

use super::{get_instance_property_error, AgalString, AgalValuable};
use crate::runtime::{env::RefEnvironment, AgalError, AgalValue, RefAgalValue, Stack};
use parser::internal::ErrorNames;

#[derive(Clone, PartialEq)]
pub enum AgalThrow {
    Params {
        type_error: ErrorNames,
        message: String,
        stack: Box<Stack>,
    },
    Error(AgalError),
}
impl AgalThrow {
    pub fn from_ref_value<T: AgalValuable>(
        v: Rc<RefCell<T>>,
        stack: Box<Stack>,
        env: RefEnvironment,
    ) -> AgalThrow {
        let str = v.borrow().clone().to_agal_console(stack.as_ref(), env);
        if str.is_err() {
            return str.err().unwrap();
        }
        let str = str.ok().unwrap();
        let message = str.get_string().clone();
        AgalThrow::Params {
            type_error: ErrorNames::None,
            message,
            stack,
        }
    }
    pub fn get_error(&self) -> AgalError {
        match self {
            AgalThrow::Params {
                type_error,
                message,
                stack,
            } => AgalError::new(type_error.clone(), message.clone(), stack.clone()),
            AgalThrow::Error(e) => e.clone(),
        }
    }
}

impl AgalValuable for AgalThrow {
    fn to_value(self) -> AgalValue {
        AgalValue::Throw(self)
    }
    fn call(
        self,
        _: &Stack,
        _: RefEnvironment,
        _: RefAgalValue,
        _: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        AgalValue::Throw(self).as_ref()
    }
    fn get_instance_property(self, _: &Stack, _: RefEnvironment, _: String) -> RefAgalValue {
        AgalValue::Throw(self).as_ref()
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        self.to_agal_string(stack, env)
    }
}

impl std::fmt::Display for AgalThrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalThrow::Params {
                type_error,
                message,
                stack,
            } => {
                write!(f, "{}: {}\n{}", type_error, message, stack)
            }
            AgalThrow::Error(_) => write!(f, "Error"),
        }
    }
}

pub struct AgalNativeFunction {
    pub name: String,
    pub func: Rc<dyn Fn(Vec<RefAgalValue>, &Stack, RefEnvironment) -> RefAgalValue>,
}
impl Clone for AgalNativeFunction {
    fn clone(&self) -> Self {
        AgalNativeFunction {
            name: self.name.clone(),
            func: self.func.clone(),
        }
    }
}
impl AgalValuable for AgalNativeFunction {
    fn to_value(self) -> AgalValue {
        AgalValue::NativeFunction(self)
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
}
