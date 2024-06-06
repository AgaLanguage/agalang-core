use crate::runtime::{env::{FALSE_KEYWORD, TRUE_KEYWORD}, AgalArray, Enviroment, Stack};

use super::{AgalThrow, AgalValuable};

pub type AgalString = AgalArray<AgalChar>;
impl AgalString {
    pub fn from_string(value: String) -> AgalString {
        AgalArray::from_vec(value.chars().map(AgalChar).collect())
    }
    pub fn get_string(&self) -> String {
        self.get_vec().iter().map(|c| c.to_char()).collect()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct AgalNumber(f64);
impl AgalNumber {
    pub fn new(value: f64) -> AgalNumber {
        AgalNumber(value)
    }
    pub fn to_f64(&self) -> f64 {
        self.0
    }
}
impl AgalValuable for AgalNumber {
    fn to_agal_number(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalNumber, AgalThrow> {
        Ok(self)
    }
    fn to_agal_boolean(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalBoolean, AgalThrow> {
        Ok(AgalBoolean(self.0 != 0f64))
    }
    fn to_agal_string(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(self.0.to_string()))
    }
    fn to_agal_console(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("\x1b[33{}\x1b[39", self.0)))
    }
}

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
impl AgalValuable for AgalChar {}

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
    fn to_agal_boolean(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalBoolean, AgalThrow> {
        Ok(self)
    }
    fn to_agal_string(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(bool_to_str(self.0)))
    }
    fn to_agal_console(
        self,
        _: Box<Stack>,
        _: &Enviroment
    ) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("\x1b[33{}\x1b[39", bool_to_str(self.0))))
    }
}