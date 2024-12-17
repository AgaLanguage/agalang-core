mod class;
use std::future::Future;

pub use class::*;
mod function;
pub use function::*;
mod object;
pub use object::*;
mod array;
pub use array::*;
mod error;
pub use error::*;
mod Promise;
pub use Promise::*;

use crate::{
    runtime::{
        env::RefEnvironment, AgalBoolean, AgalByte, AgalNumber, AgalString, AgalThrow,
        AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
    },
    Modules,
};

pub enum AgalComplex<'a> {
    Array(AgalArray<'a>),
    Class(AgalClass<'a>),
    Error(AgalError),
    Function(AgalFunction),
    Object(AgalObject<'a>),
    SuperInstance(AgalPrototype<'a>),
    Promise(AgalPromise<'a>),
}

impl<'a> PartialEq for AgalComplex<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Class(c), Self::Class(d)) => c == d,
            (Self::Error(e), Self::Error(f)) => e == f,
            (Self::Function(f), Self::Function(g)) => f == g,
            (Self::Object(h), Self::Object(i)) => h == i,
            (Self::SuperInstance(j), Self::SuperInstance(k)) => j == k,
            _ => false,
        }
    }
}

impl<'a> AgalValuableManager<'a> for AgalComplex<'a> {
    fn to_value(self) -> AgalValue<'a> {
        AgalValue::Complex(self)
    }
    fn get_type(&self) -> &'static str {
        match self {
            Self::Array(_) => "Arreglo",
            Self::Class(_) => "Clase",
            Self::Error(_) => "Error",
            Self::Function(_) => "Funcion",
            Self::Object(_) => "Objeto",
            Self::SuperInstance(_) => "Instancia super",
            Self::Promise(_) => "Promesa",
        }
    }

    fn get_keys(&self) -> Vec<String> {
        match self {
            Self::Array(a) => a.get_keys(),
            Self::Class(c) => c.get_keys(),
            Self::Error(e) => e.get_keys(),
            Self::Function(f) => f.get_keys(),
            Self::Object(o) => o.get_keys(),
            Self::SuperInstance(p) => p.get_keys(),
            Self::Promise(p) => p.get_keys(),
        }
    }

    fn get_length(&self) -> usize {
        match self {
            Self::Array(a) => a.get_length(),
            Self::Class(c) => c.get_length(),
            Self::Error(e) => e.get_length(),
            Self::Function(f) => f.get_length(),
            Self::Object(o) => o.get_length(),
            Self::SuperInstance(p) => p.get_length(),
            Self::Promise(p) => p.get_length(),
        }
    }

    fn to_agal_number(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_number(stack, env),
            Self::Class(c) => c.to_agal_number(stack, env),
            Self::Error(e) => e.to_agal_number(stack, env),
            Self::Function(f) => f.to_agal_number(stack, env),
            Self::Object(o) => o.to_agal_number(stack, env),
            Self::SuperInstance(p) => p.to_agal_number(stack, env),
            Self::Promise(p) => p.to_agal_number(stack, env),
        }
    }

    fn to_agal_string(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_string(stack, env),
            Self::Class(c) => c.to_agal_string(stack, env),
            Self::Error(e) => e.to_agal_string(stack, env),
            Self::Function(f) => f.to_agal_string(stack, env),
            Self::Object(o) => o.to_agal_string(stack, env),
            Self::SuperInstance(p) => p.to_agal_string(stack, env),
            Self::Promise(p) => p.to_agal_string(stack, env),
        }
    }

    fn to_agal_boolean(&self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalBoolean, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_boolean(stack, env),
            Self::Class(c) => c.to_agal_boolean(stack, env),
            Self::Error(e) => e.to_agal_boolean(stack, env),
            Self::Function(f) => f.to_agal_boolean(stack, env),
            Self::Object(o) => o.to_agal_boolean(stack, env),
            Self::SuperInstance(p) => p.to_agal_boolean(stack, env),
            Self::Promise(p) => p.to_agal_boolean(stack, env),
        }
    }

    fn to_agal_array(&self, stack: &Stack) -> Result<&AgalArray<'a>, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_array(stack),
            Self::Class(c) => c.to_agal_array(stack),
            Self::Error(e) => e.to_agal_array(stack),
            Self::Function(f) => f.to_agal_array(stack),
            Self::Object(o) => o.to_agal_array(stack),
            Self::SuperInstance(s) => s.to_agal_array(stack),
            Self::Promise(p) => p.to_agal_array(stack),
        }
    }

    fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_byte(stack),
            Self::Class(c) => c.to_agal_byte(stack),
            Self::Error(e) => e.to_agal_byte(stack),
            Self::Function(f) => f.to_agal_byte(stack),
            Self::Object(o) => o.to_agal_byte(stack),
            Self::SuperInstance(s) => s.to_agal_byte(stack),
            Self::Promise(p) => p.to_agal_byte(stack),
        }
    }

    fn to_agal_value(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_value(stack, env),
            Self::Class(c) => c.to_agal_value(stack, env),
            Self::Error(e) => e.to_agal_value(stack, env),
            Self::Function(f) => f.to_agal_value(stack, env),
            Self::Object(o) => o.to_agal_value(stack, env),
            Self::SuperInstance(s) => s.to_agal_value(stack, env),
            Self::Promise(p) => p.to_agal_value(stack, env),
        }
    }

    fn to_agal_console(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_console(stack, env),
            Self::Class(c) => c.to_agal_console(stack, env),
            Self::Error(e) => e.to_agal_console(stack, env),
            Self::Function(f) => f.to_agal_console(stack, env),
            Self::Object(o) => o.to_agal_console(stack, env),
            Self::SuperInstance(s) => s.to_agal_console(stack, env),
            Self::Promise(p) => p.to_agal_console(stack, env),
        }
    }

    fn binary_operation(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        operator: &str,
        other: RefAgalValue<'a>,
    ) -> RefAgalValue<'a> {
        match self {
            Self::Array(a) => a.binary_operation(stack, env, operator, other),
            Self::Class(c) => c.binary_operation(stack, env, operator, other),
            Self::Error(e) => e.binary_operation(stack, env, operator, other),
            Self::Function(f) => f.binary_operation(stack, env, operator, other),
            Self::Object(o) => o.binary_operation(stack, env, operator, other),
            Self::SuperInstance(s) => s.binary_operation(stack, env, operator, other),
            Self::Promise(p) => p.binary_operation(stack, env, operator, other),
        }
    }

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match self {
            Self::Array(a) => a.unary_operator(stack, env, operator),
            Self::Class(c) => c.unary_operator(stack, env, operator),
            Self::Error(e) => e.unary_operator(stack, env, operator),
            Self::Function(f) => f.unary_operator(stack, env, operator),
            Self::Object(o) => o.unary_operator(stack, env, operator),
            Self::SuperInstance(s) => s.unary_operator(stack, env, operator),
            Self::Promise(p) => p.unary_operator(stack, env, operator),
        }
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match self {
            Self::Array(a) => a.unary_back_operator(stack, env, operator),
            Self::Class(c) => c.unary_back_operator(stack, env, operator),
            Self::Error(e) => e.unary_back_operator(stack, env, operator),
            Self::Function(f) => f.unary_back_operator(stack, env, operator),
            Self::Object(o) => o.unary_back_operator(stack, env, operator),
            Self::SuperInstance(s) => s.unary_back_operator(stack, env, operator),
            Self::Promise(p) => p.unary_back_operator(stack, env, operator),
        }
    }

    fn get_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) -> RefAgalValue<'a> {
        match self {
            Self::Array(a) => a.get_object_property(stack, env, key),
            Self::Class(c) => c.get_object_property(stack, env, key),
            Self::Error(e) => e.get_object_property(stack, env, key),
            Self::Function(f) => f.get_object_property(stack, env, key),
            Self::Object(o) => o.get_object_property(stack, env, key),
            Self::SuperInstance(s) => s.get_object_property(stack, env, key),
            Self::Promise(p) => p.get_object_property(stack, env, key),
        }
    }

    fn set_object_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        match self {
            Self::Array(a) => a.set_object_property(stack, env, key, value),
            Self::Class(c) => c.set_object_property(stack, env, key, value),
            Self::Error(e) => e.set_object_property(stack, env, key, value),
            Self::Function(f) => f.set_object_property(stack, env, key, value),
            Self::Object(o) => o.set_object_property(stack, env, key, value),
            Self::SuperInstance(s) => s.set_object_property(stack, env, key, value),
            Self::Promise(p) => p.set_object_property(stack, env, key, value),
        }
    }

    fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment, key: String) {
        match self {
            Self::Array(a) => a.delete_object_property(stack, env, key),
            Self::Class(c) => c.delete_object_property(stack, env, key),
            Self::Error(e) => e.delete_object_property(stack, env, key),
            Self::Function(f) => f.delete_object_property(stack, env, key),
            Self::Object(o) => o.delete_object_property(stack, env, key),
            Self::SuperInstance(s) => s.delete_object_property(stack, env, key),
            Self::Promise(p) => p.delete_object_property(stack, env, key),
        }
    }

    fn get_instance_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
    ) -> super::RefAgalValue<'a> {
        match self {
            Self::Array(a) => a.get_instance_property(stack, env, key),
            Self::Class(c) => c.get_instance_property(stack, env, key),
            Self::Error(e) => e.get_instance_property(stack, env, key),
            Self::Function(f) => f.get_instance_property(stack, env, key),
            Self::Object(o) => o.get_instance_property(stack, env, key),
            Self::SuperInstance(s) => s.get_instance_property(stack, env, key),
            Self::Promise(p) => p.get_instance_property(stack, env, key),
        }
    }

    async fn call(
        &self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        this: RefAgalValue<'a>,
        args: Vec<RefAgalValue<'a>>,
        modules: &Modules<'a>,
    ) -> RefAgalValue<'a> {
        match self {
            Self::Array(a) => a.call(stack, env, this, args, modules).await,
            Self::Class(c) => c.call(stack, env, this, args, modules).await,
            Self::Error(e) => e.call(stack, env, this, args, modules).await,
            Self::Function(f) => f.call(stack, env, this, args, modules).await,
            Self::Object(o) => o.call(stack, env, this, args, modules).await,
            Self::SuperInstance(s) => s.call(stack, env, this, args, modules).await,
            Self::Promise(p) => p.call(stack, env, this, args, modules).await,
        }
    }
}
