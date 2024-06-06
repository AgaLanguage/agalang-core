use crate::{
    internal::ErrorNames,
    runtime::{AgalString, Enviroment, Stack},
};

use super::{AgalThrow, AgalValuable, AgalValue};
#[derive(Clone)]
pub struct AgalArray<T: AgalValuable>(Vec<T>);
impl<T: AgalValuable> AgalArray<T> {
    pub fn from_vec(vec: Vec<T>) -> AgalArray<T> {
        AgalArray(vec)
    }
    pub fn get_vec(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T: AgalValuable> AgalValuable<T> for AgalArray<T> {
    fn to_agal_console(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.0.iter() {
            let str = value.clone().to_agal_value(stack.clone(), env);
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str
                .get_vec()
                .iter()
                .map(|c| c.to_char())
                .collect::<String>();
            result.push_str(&str);
        }
        Ok(AgalString::from_string(result))
    }
    fn to_agal_array(self, _: Box<Stack>) -> Result<AgalArray<T>, AgalThrow> {
        Ok(self)
    }
}

#[derive(Clone)]
pub struct AgalObject(std::collections::HashMap<String, AgalValue>);
impl AgalValuable for AgalObject {
    fn to_agal_console(self, _: Box<Stack>, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        let string = "\x1b[36m[Objeto]\x1b[39m".to_string();
        Ok(AgalString::from_string(string))
    }
}
#[derive(Clone)]
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
}
impl AgalValuable for AgalError {
    fn to_agal_string(self, _: Box<Stack>, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "{}: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_value(self, _: Box<Stack>, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[91m{}\x1b[39m: {}",
            self.type_error, self.message
        )))
    }
    fn to_agal_console(self, _: Box<Stack>, _: &Enviroment) -> Result<AgalString, AgalThrow> {
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
}
