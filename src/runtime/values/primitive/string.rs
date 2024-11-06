use std::rc::Rc;

use crate::runtime::{
    binary_operation_error,
    env::RefEnvironment,
    get_instance_property_error, unary_operation_error,
    values::{AgalNumber, AgalThrow, AgalValuable, AgalValue},
    AgalArray, RefAgalValue, Stack,
};

use super::AgalBoolean;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalString(String);
impl AgalString {
    pub fn from_string(value: String) -> AgalString {
        AgalString(value)
    }
    pub fn get_string(&self) -> &String {
        &self.0
    }
    pub fn string_to_array(value: &AgalString) -> AgalArray {
        get_chars(value)
    }
}
impl AgalValuable for AgalString {
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(self)
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("{}", self.0)))
    }
    fn to_agal_array(self, _: &Stack) -> Result<AgalArray, AgalThrow> {
        Ok(AgalString::string_to_array(&self))
    }
    fn to_agal_value(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[32m'{}'\x1b[39m",
            self.0
                .replace("'", "\\'")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t")
        )))
    }
    fn to_value(self) -> AgalValue {
        AgalValue::String(self)
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        match key.as_str() {
            "caracteres" => {
                let function = move |_: Vec<RefAgalValue>, _: &Stack, _: RefEnvironment| {
                    AgalValue::Array(get_chars(&self)).as_ref()
                };
                let func = Rc::new(function);
                AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                    name: "caracteres".to_string(),
                    func,
                })
                .as_ref()
            }
            "largo" => get_length(&self).as_ref(),
            _ => {
                let value = AgalValue::String(self);
                get_instance_property_error(stack, env, key, value)
            }
        }
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        _: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        let cself = self.clone();
        let cother = other.clone();
        let other: &AgalValue = &other.borrow();
        match other {
            AgalValue::String(other) => match operator {
                "+" => {
                    let mut new_string = self.get_string().clone();
                    new_string.push_str(other.get_string());
                    AgalValue::String(AgalString::from_string(new_string)).as_ref()
                }
                "==" => AgalValue::Boolean(AgalBoolean(self.0 == other.0)).as_ref(),
                "!=" => AgalValue::Boolean(AgalBoolean(self.0 != other.0)).as_ref(),
                "&&" => {
                    return (if self.0 == "" { self } else { other })
                        .clone()
                        .to_value()
                        .as_ref()
                }
                "||" => {
                    return (if self.0 != "" { self } else { other })
                        .clone()
                        .to_value()
                        .as_ref()
                }
                _ => binary_operation_error(stack, operator, cself.to_value().as_ref(), cother),
            },
            _ => binary_operation_error(stack, operator, cself.to_value().as_ref(), cother),
        }
    }
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match operator {
            "?" => match self.clone().to_agal_boolean(stack, env) {
                Ok(bool) => bool.to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "!" => match self.clone().to_agal_boolean(stack, env) {
                Ok(bool) => AgalBoolean::new(!bool.to_bool()).to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "+" => match self.clone().to_agal_number(stack, env) {
                Ok(num) => num.to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "&" => self.clone().to_ref_value(),
            _ => unary_operation_error(stack, operator, self.clone().to_ref_value()),
        }
    }
}

// instance methods

fn get_chars(value: &AgalString) -> AgalArray {
    let vec = value.get_string();
    let mut new_vec: Vec<RefAgalValue> = vec![];

    for c in vec.chars() {
        let i = AgalValue::Char(AgalChar::new(c)).as_ref();
        new_vec.push(i);
    }
    AgalArray::from_vec(new_vec)
}
fn get_length(value: &AgalString) -> AgalValue {
    AgalValue::Number(AgalNumber::new(value.get_string().len() as f64))
}

/**
 * Agal Character are a single character
 */

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalChar(char);
impl AgalChar {
    pub fn new(value: char) -> AgalChar {
        AgalChar(value)
    }
    pub fn to_char(&self) -> char {
        self.0
    }
}
impl AgalValuable for AgalChar {
    fn to_value(self) -> AgalValue {
        AgalValue::Char(self)
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[34m'{}'\x1b[39m",
            match self.0 {
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\'' => "\\'".to_string(),
                _ => format!("{}", self.0),
            }
        )))
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let value = AgalValue::Char(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(self.0.to_string()))
    }
}
