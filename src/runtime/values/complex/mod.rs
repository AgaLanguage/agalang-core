use crate::runtime::{env::RefEnvironment, AgalString, Stack};
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

use super::{
    get_instance_property_error, get_property_error, AgalThrow, AgalValuable, AgalValue,
    RefAgalValue,
};

pub type AgalVec = Vec<Rc<RefCell<AgalValue>>>;
#[derive(Clone, PartialEq)]
pub struct AgalArray(AgalVec);
impl AgalArray {
    pub fn from_vec(vec: AgalVec) -> AgalArray {
        AgalArray(vec)
    }
    pub fn get_vec(&self) -> &AgalVec {
        &self.0
    }
}
impl AgalValuable for AgalArray {
    fn get_length(self) -> usize {
        self.0.len()
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.0.iter() {
            let str = value
                .as_ref()
                .borrow()
                .clone()
                .to_agal_value(stack, env.clone());
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str.get_string();
            result.push_str(&str);
        }
        Ok(AgalString::from_string(result))
    }
    fn to_agal_array(self, _: &Stack) -> Result<AgalArray, AgalThrow> {
        Ok(self)
    }
    fn to_value(self) -> AgalValue {
        AgalValue::Array(self)
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.0.iter() {
            let str = value.as_ref().borrow().clone().to_agal_string(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str.get_string();
            result.push_str(&str);
        }
        Ok(AgalString::from_string(result))
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let value = AgalValue::Array(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        let int = key.parse::<usize>();
        if int.is_err() {
            return get_property_error(stack, env, key);
        }
        let int = int.unwrap();
        let value = self.0.get(int);
        value.unwrap_or(&AgalValue::Never.as_ref()).clone()
    }
}

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
            stack.push_str(&format!("{}:{}", frame.get_file(), frame.get_location().start.line));
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
