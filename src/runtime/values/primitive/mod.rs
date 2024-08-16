use crate::runtime::{env::{RefEnviroment, FALSE_KEYWORD, TRUE_KEYWORD}, Stack};

use super::{super::{AgalThrow, AgalValuable, AgalValue}, get_instance_property_error, RefAgalValue};

mod string;
pub use string::{AgalString, AgalChar};
mod byte;
pub use byte::AgalByte;


#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct AgalNumber(f64);
impl AgalNumber {
    pub fn new(value: f64) -> AgalNumber {
        AgalNumber(value)
    }
    pub fn to_f64(&self) -> f64 {
        self.0
    }
    pub fn multiply(&self, other: AgalNumber) -> AgalNumber {
        AgalNumber::new(self.0 * other.0)
    }
}
impl AgalValuable for AgalNumber {
    fn to_value(self) -> AgalValue {
        AgalValue::Number(self)
    }
    fn to_agal_number(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalNumber, AgalThrow> {
        Ok(self)
    }
    fn to_agal_boolean(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalBoolean, AgalThrow> {
        Ok(AgalBoolean(self.0 != 0f64))
    }
    fn to_agal_string(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(self.0.to_string()))
    }
    fn to_agal_console(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("\x1b[33{}\x1b[39", self.0)))
    }
    fn get_instance_property(self, stack: &Stack, env: RefEnviroment, key: String) -> RefAgalValue {
        let value = AgalValue::Number(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn call(self, stack: &Stack, env: RefEnviroment, _: RefAgalValue, list: Vec<RefAgalValue>) -> RefAgalValue {
        let value = list.get(0);
        if value.is_none() {
            return AgalValue::Number(self).as_ref();
        }
        let value = value.unwrap();
        let other = value.borrow().clone().to_agal_number(stack, env);
        if other.is_err() {
            return other.err().unwrap().to_value().as_ref();
        }
        let other = other.ok().unwrap();
        let number = self.multiply(other);
        AgalValue::Number(number).as_ref()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalBoolean(bool);
impl AgalBoolean {
    pub fn new(value: bool) -> AgalBoolean {
        AgalBoolean(value)
    }
    pub fn to_bool(&self) -> bool {
        self.0
    }
}
fn bool_to_str(value: bool) -> String {
    let data = if value {
        TRUE_KEYWORD
    } else {
        FALSE_KEYWORD
    };
    data.to_string()
}
impl AgalValuable for AgalBoolean {
    fn to_value(self) -> AgalValue {
        AgalValue::Boolean(self)
    }
    fn to_agal_boolean(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalBoolean, AgalThrow> {
        Ok(self)
    }
    fn to_agal_string(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(bool_to_str(self.0)))
    }
    fn to_agal_console(
        self,
        _: &Stack,
        _: RefEnviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("\x1b[33{}\x1b[39", bool_to_str(self.0))))
    }
    fn get_instance_property(self, stack: &Stack, env: RefEnviroment, key: String) -> RefAgalValue {
        let value = AgalValue::Boolean(self);
        get_instance_property_error(stack, env, key, value)
    }
}