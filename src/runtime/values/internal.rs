use super::AgalValuable;
use crate::{internal::ErrorNames, runtime::{AgalError, Stack}};

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
    pub fn from_value<T: AgalValuable>(v: T, stack: Box<Stack>, env: &mut crate::runtime::Enviroment) -> AgalThrow {
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
    fn to_value(self) -> super::AgalValue {
        super::AgalValue::Throw(self)
    }
    fn call(self, _: &Stack, _: &crate::runtime::Enviroment, _: super::AgalValue, _: Vec<super::AgalValue>) -> super::AgalValue {
        super::AgalValue::Throw(self)
    }
    fn get_instance_property(self, _: &Stack, _: &super::Enviroment, _: String) -> super::AgalValue {
        super::AgalValue::Throw(self)
    }
}

impl std::fmt::Display for AgalThrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalThrow::Params { type_error, message, stack } => {
                write!(f, "{}: {}\n{}", type_error, message, stack)
            }
            AgalThrow::Error(e) => write!(f, "Error"),
        }
    }
}
