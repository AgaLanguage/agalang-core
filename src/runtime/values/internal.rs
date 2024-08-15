use std::{cell::RefCell, rc::Rc};

use super::{get_instance_property_error, AgalString, AgalValuable};
use crate::{internal::ErrorNames, runtime::{AgalError, Stack, AgalValue, Enviroment, RefAgalValue}};

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
    pub fn new(type_error: ErrorNames, message: String, stack: Box<Stack>) -> AgalThrow {
        AgalThrow::Params {
            type_error,
            message,
            stack,
        }
    }
    pub fn from_value<T: AgalValuable>(v: T, stack: Box<Stack>, env: &mut Enviroment) -> AgalThrow {
        let str = v.to_agal_console(stack.as_ref(), env);
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
    pub fn from_ref_value<T: AgalValuable>(v: Rc<RefCell<T>>, stack: Box<Stack>, env: &mut Enviroment) -> AgalThrow {
        let str = v.as_ref().borrow().clone().to_agal_console(stack.as_ref(), env);
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
    pub fn from_error(v: AgalError) -> AgalThrow {
        AgalThrow::Error(v)
    }
    pub fn get_error(&self) -> AgalError {
        match self {
            AgalThrow::Params { type_error, message, stack } => AgalError::new(type_error.clone(), message.clone(), stack.clone()),
            AgalThrow::Error(e) => e.clone(),
        }
    }
}

impl AgalValuable for AgalThrow {
    fn to_value(self) -> AgalValue {
        AgalValue::Throw(self)
    }
    fn call(self, _: &Stack, _: &crate::runtime::Enviroment, _: RefAgalValue, _: Vec<RefAgalValue>) -> RefAgalValue {
        AgalValue::Throw(self).to_ref()
    }
    fn get_instance_property(self, _: &Stack, _: &Enviroment, _: String) -> RefAgalValue {
        AgalValue::Throw(self).to_ref()
    }
}

impl std::fmt::Display for AgalThrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalThrow::Params { type_error, message, stack } => {
                write!(f, "{}: {}\n{}", type_error, message, stack)
            }
            AgalThrow::Error(_) => write!(f, "Error"),
        }
    }
}

pub struct AgalNativeFunction {
    pub name: String,
    pub func: Rc<dyn Fn(Vec<RefAgalValue>) -> RefAgalValue>,
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
    fn to_agal_string(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("<Funcion nativa {}>", self.name)))
    }
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
        get_instance_property_error(stack, env, key, self.to_value())
    }
}