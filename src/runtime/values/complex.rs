use crate::{
    frontend::{node_error, ast::NodeError}, internal::{ErrorTypes, ErrorNames}, runtime::{AgalString, Enviroment, Stack}
};

use super::{get_instance_property_error, AgalThrow, AgalValuable, AgalValue};

#[derive(Clone, PartialEq)]
pub struct AgalArray(Vec<AgalValue>);
impl AgalArray {
    pub fn from_vec(vec: Vec<AgalValue>) -> AgalArray {
        AgalArray(vec)
    }
    pub fn get_vec(&self) -> &Vec<AgalValue> {
        &self.0
    }
}

impl AgalValuable for AgalArray {
    fn to_agal_console(self, stack: &Stack, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.0.iter() {
            let str = value.clone().to_agal_value(stack, env);
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str
                .get_string();
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
    fn to_agal_string(self, stack: &Stack, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.0.iter() {
            let str = value.clone().to_agal_string(stack, &env.clone().crate_child());
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str.get_string();
            result.push_str(&str);
        }
        Ok(AgalString::from_string(result))
    }
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> AgalValue {
        let value = AgalValue::Array(self);
        get_instance_property_error(stack, env, key, value)
    }
}

#[derive(Clone, PartialEq)]
pub struct AgalObject(std::collections::HashMap<String, AgalValue>);
impl AgalValuable for AgalObject {
    fn to_agal_console(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        let string = "\x1b[36m[Objeto]\x1b[39m".to_string();
        Ok(AgalString::from_string(string))
    }
    fn to_value(self) -> AgalValue {
        AgalValue::Object(self)
    }
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> AgalValue {
        let value = AgalValue::Object(self);
        get_instance_property_error(stack, env, key, value)
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
    fn to_agal_string(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "{}: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_value(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[91m{}\x1b[39m: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_console(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        let error = format!("\x1b[91m{}\x1b[39m: {}", self.type_error, self.message);
        let mut stack = String::new();
        let stack_vec = self.stack.iter();
        for (i, frame) in stack_vec.iter().enumerate() {
            stack.push_str(&format!("{}:{}", frame.get_file(), frame.get_line()));
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
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> AgalValue {
        let value = AgalValue::Error(self);
        get_instance_property_error(stack, env, key, value)
    }
}
