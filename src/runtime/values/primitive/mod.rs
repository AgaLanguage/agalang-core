use crate::runtime::{
    env::{RefEnvironment, FALSE_KEYWORD, TRUE_KEYWORD},
    get_instance_property_error, unary_operation_error, AgalArray, AgalThrow, AgalValuable,
    AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

mod string;
pub use string::{AgalChar, AgalString};
mod byte;
pub use byte::AgalByte;
mod number;
pub use number::*;
mod boolean;
pub use boolean::*;

#[derive(Clone, PartialEq)]
pub enum AgalPrimitive<'a> {
    String(AgalString<'a>),
    Char(AgalChar),
    Byte(AgalByte),
    Number(AgalNumber),
    Boolean(AgalBoolean),
}

impl<'a> AgalValuableManager<'a> for AgalPrimitive<'a> {
    fn get_type(&self) -> &'static str {
        match self {
            Self::String(_) => "Cadena",
            Self::Char(_) => "Caracter",
            Self::Byte(_) => "Byte",
            Self::Number(_) => "Numero",
            Self::Boolean(_) => "Buleano",
        }
    }

    fn to_value(self) -> AgalValue<'a> {
        AgalValue::Primitive(self)
    }

    fn get_keys(&self) -> Vec<String> {
        match self {
            Self::String(s) => s.get_keys(),
            Self::Char(c) => c.get_keys(),
            Self::Byte(b) => b.get_keys(),
            Self::Number(n) => n.get_keys(),
            Self::Boolean(b) => b.get_keys(),
        }
    }

    fn get_length(&self) -> usize {
        match self {
            Self::String(s) => s.get_length(),
            Self::Char(c) => c.get_length(),
            Self::Byte(b) => b.get_length(),
            Self::Number(n) => n.get_length(),
            Self::Boolean(b) => b.get_length(),
        }
    }

    fn to_agal_number(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_number(stack, env),
            Self::Char(c) => c.to_agal_number(stack, env),
            Self::Byte(b) => b.to_agal_number(stack, env),
            Self::Number(n) => Ok(n.clone()),
            Self::Boolean(b) => b.to_agal_number(stack, env),
        }
    }

    fn to_agal_string(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString, AgalThrow> {
        match self {
            Self::String(s) => Ok(s.clone()),
            Self::Char(c) => c.to_agal_string(stack, env),
            Self::Byte(b) => b.to_agal_string(stack, env),
            Self::Number(n) => n.to_agal_string(stack, env),
            Self::Boolean(b) => b.to_agal_string(stack, env),
        }
    }

    fn to_agal_boolean(&self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalBoolean, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_boolean(stack, env),
            Self::Char(c) => c.to_agal_boolean(stack, env),
            Self::Byte(b) => b.to_agal_boolean(stack, env),
            Self::Number(n) => n.to_agal_boolean(stack, env),
            Self::Boolean(b) => Ok(b.clone()),
        }
    }

    fn to_agal_array(&self, stack: &Stack) -> Result<&AgalArray<'a>, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_array(stack),
            Self::Char(c) => c.to_agal_array(stack),
            Self::Byte(b) => b.to_agal_array(stack),
            Self::Number(n) => n.to_agal_array(stack),
            Self::Boolean(b) => b.to_agal_array(stack),
        }
    }

    fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_byte(stack),
            Self::Char(c) => c.to_agal_byte(stack),
            Self::Byte(b) => Ok(b.clone()),
            Self::Number(n) => n.to_agal_byte(stack),
            Self::Boolean(b) => b.to_agal_byte(stack),
        }
    }

    fn to_agal_value(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_value(stack, env),
            Self::Char(c) => c.to_agal_value(stack, env),
            Self::Byte(b) => b.to_agal_value(stack, env),
            Self::Number(n) => n.to_agal_value(stack, env),
            Self::Boolean(b) => b.to_agal_value(stack, env),
        }
    }

    fn to_agal_console(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString, AgalThrow> {
        match self {
            Self::String(s) => s.to_agal_console(stack, env),
            Self::Char(c) => c.to_agal_console(stack, env),
            Self::Byte(b) => b.to_agal_console(stack, env),
            Self::Number(n) => n.to_agal_console(stack, env),
            Self::Boolean(b) => b.to_agal_console(stack, env),
        }
    }

    fn binary_operation(
        &'a self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue<'a>,
    ) -> RefAgalValue {
        match self {
            Self::String(s) => s.binary_operation(stack, env, operator, other),
            Self::Char(c) => c.binary_operation(stack, env, operator, other),
            Self::Byte(b) => b.binary_operation(stack, env, operator, other),
            Self::Number(n) => n.binary_operation(stack, env, operator, other),
            Self::Boolean(b) => b.binary_operation(stack, env, operator, other),
        }
    }

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match self {
            Self::String(s) => s.unary_operator(stack, env, operator),
            Self::Char(c) => c.unary_operator(stack, env, operator),
            Self::Byte(b) => b.unary_operator(stack, env, operator),
            Self::Number(n) => n.unary_operator(stack, env, operator),
            Self::Boolean(b) => b.unary_operator(stack, env, operator),
        }
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match self {
            Self::String(s) => s.unary_back_operator(stack, env, operator),
            Self::Char(c) => c.unary_back_operator(stack, env, operator),
            Self::Byte(b) => b.unary_back_operator(stack, env, operator),
            Self::Number(n) => n.unary_back_operator(stack, env, operator),
            Self::Boolean(b) => b.unary_back_operator(stack, env, operator),
        }
    }

    fn get_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) -> RefAgalValue {
        match self {
            Self::String(s) => s.get_object_property(stack, env, key),
            Self::Char(c) => c.get_object_property(stack, env, key),
            Self::Byte(b) => b.get_object_property(stack, env, key),
            Self::Number(n) => n.get_object_property(stack, env, key),
            Self::Boolean(b) => b.get_object_property(stack, env, key),
        }
    }

    fn set_object_property(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        match self {
            Self::String(s) => s.set_object_property(stack, env, key, value),
            Self::Char(c) => c.set_object_property(stack, env, key, value),
            Self::Byte(b) => b.set_object_property(stack, env, key, value),
            Self::Number(n) => n.set_object_property(stack, env, key, value),
            Self::Boolean(b) => b.set_object_property(stack, env, key, value),
        }
    }

    fn delete_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) {
        match self {
            Self::String(s) => s.delete_object_property(stack, env, key),
            Self::Char(c) => c.delete_object_property(stack, env, key),
            Self::Byte(b) => b.delete_object_property(stack, env, key),
            Self::Number(n) => n.delete_object_property(stack, env, key),
            Self::Boolean(b) => b.delete_object_property(stack, env, key),
        }
    }

    fn get_instance_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
    ) -> RefAgalValue {
        match self {
            Self::String(s) => s.get_instance_property(stack, env, key),
            Self::Char(c) => c.get_instance_property(stack, env, key),
            Self::Byte(b) => b.get_instance_property(stack, env, key),
            Self::Number(n) => n.get_instance_property(stack, env, key),
            Self::Boolean(b) => b.get_instance_property(stack, env, key),
        }
    }

    async fn call(
        &self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        this: RefAgalValue<'a>,
        args: Vec<RefAgalValue<'a>>,
        modules: &crate::Modules<'a>,
    ) -> RefAgalValue<'a> {
        match self {
            Self::String(s) => s.call(stack, env, this, args, modules).await,
            Self::Char(c) => c.call(stack, env, this, args, modules).await,
            Self::Byte(b) => b.call(stack, env, this, args, modules).await,
            Self::Number(n) => n.call(stack, env, this, args, modules).await,
            Self::Boolean(b) => b.call(stack, env, this, args, modules).await,
        }
    }
}
