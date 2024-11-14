use crate::{
    runtime::{env::RefEnvironment, AgalString, Stack},
    Modules,
};
use parser::{
    internal::{ErrorNames, ErrorTypes},
    node_error,
};
use std::{cell::RefCell, rc::Rc};
mod class;
pub use class::*;
mod function;
pub use function::*;
mod object;
pub use object::*;
mod array;
pub use array::*;

use super::{
    get_instance_property_error, get_property_error, AgalByte, AgalNumber, AgalThrow, AgalValuable,
    AgalValue, RefAgalValue,
};
#[derive(Clone, PartialEq)]
pub struct AgalError {
    type_error: ErrorNames,
    message: String,
    stack: Box<Stack>,
}
impl AgalError {
    pub fn new(type_error: ErrorNames, message: String, stack: Box<Stack>) -> AgalError {
        AgalError {
            type_error,
            message,
            stack,
        }
    }
    pub fn get_type_error(&self) -> ErrorNames {
        self.type_error.clone()
    }
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
    pub fn to_error(&self) -> ErrorTypes {
        let value = self.stack.get_value();
        let error = if value.is_error() {
            value.get_error().unwrap().clone()
        } else {
            return ErrorTypes::StringError(format!("{}", self.get_message()));
        };
        node_error(&error)
    }
}
impl AgalValuable for AgalError {
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "{}: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_value(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[91m{}\x1b[39m: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let error = format!("\x1b[91m{}\x1b[39m: {}", self.type_error, self.message);
        let mut stack = String::new();
        let stack_vec = self.stack.iter();
        for (i, frame) in stack_vec.iter().enumerate() {
            stack.push_str(&format!(
                "{}:{}",
                frame.get_file(),
                frame.get_location().start.line
            ));
            if i < stack_vec.len() - 1 {
                stack.push_str(" -> ");
            }
        }
        let stack = format!("\x1b[90m{}\x1b[39m", stack);
        Ok(AgalString::from_string(format!("{} {}", error, stack)))
    }
    fn to_value(self) -> AgalValue {
        AgalValue::Error(self)
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let value = AgalValue::Error(self);
        get_instance_property_error(stack, env, key, value)
    }
}
