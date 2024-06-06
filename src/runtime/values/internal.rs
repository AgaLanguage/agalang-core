use super::AgalValuable;
use crate::{internal::ErrorNames, runtime::{AgalError, Stack}};
#[derive(Clone)]
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
        let str = v.to_agal_console(stack.clone(), env);
        if str.is_err() {
            return str.err().unwrap();
        }
        let str = str.ok().unwrap();
        let message = str.get_vec().iter().map(|c| c.to_char()).collect::<String>();
        AgalThrow::Params {
            type_error: ErrorNames::None,
            message,
            stack,
        }
    }
    pub fn from_error(v: AgalError) -> AgalThrow {
        AgalThrow::Error(v)
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
