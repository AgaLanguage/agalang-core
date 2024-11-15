mod class;
pub use class::*;
mod function;
pub use function::*;
mod object;
pub use object::*;
mod array;
pub use array::*;
mod error;
pub use error::*;

use crate::{
    runtime::{
        env::RefEnvironment, AgalBoolean, AgalByte, AgalNumber, AgalString, AgalThrow,
        AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
    },
    Modules,
};

#[derive(Clone, PartialEq)]
pub enum AgalComplex {
    Array(AgalArray),
    Class(AgalClass),
    Error(AgalError),
    Function(AgalFunction),
    Object(AgalObject),
    SuperInstance(AgalPrototype),
}

impl AgalValuableManager for AgalComplex {
    fn to_value(self) -> AgalValue {
        AgalValue::Complex(self)
    }
    fn get_type(self) -> &'static str {
        match self {
            Self::Array(_) => "Arreglo",
            Self::Class(_) => "Clase",
            Self::Error(_) => "Error",
            Self::Function(_) => "Funcion",
            Self::Object(_) => "Objeto",
            Self::SuperInstance(_) => "Instancia super",
        }
    }

    fn get_keys(self) -> Vec<String> {
        match self {
            Self::Array(a) => a.get_keys(),
            Self::Class(c) => c.get_keys(),
            Self::Error(e) => e.get_keys(),
            Self::Function(f) => f.get_keys(),
            Self::Object(o) => o.get_keys(),
            Self::SuperInstance(p) => p.get_keys(),
        }
    }

    fn get_length(self) -> usize {
        match self {
            Self::Array(a) => a.get_length(),
            Self::Class(c) => c.get_length(),
            Self::Error(e) => e.get_length(),
            Self::Function(f) => f.get_length(),
            Self::Object(o) => o.get_length(),
            Self::SuperInstance(p) => p.get_length(),
        }
    }

    fn to_agal_number(self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_number(stack, env),
            Self::Class(c) => c.to_agal_number(stack, env),
            Self::Error(e) => e.to_agal_number(stack, env),
            Self::Function(f) => f.to_agal_number(stack, env),
            Self::Object(o) => o.to_agal_number(stack, env),
            Self::SuperInstance(p) => p.to_agal_number(stack, env),
        }
    }

    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_string(stack, env),
            Self::Class(c) => c.to_agal_string(stack, env),
            Self::Error(e) => e.to_agal_string(stack, env),
            Self::Function(f) => f.to_agal_string(stack, env),
            Self::Object(o) => o.to_agal_string(stack, env),
            Self::SuperInstance(p) => p.to_agal_string(stack, env),
        }
    }

    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_boolean(stack, env),
            Self::Class(c) => c.to_agal_boolean(stack, env),
            Self::Error(e) => e.to_agal_boolean(stack, env),
            Self::Function(f) => f.to_agal_boolean(stack, env),
            Self::Object(o) => o.to_agal_boolean(stack, env),
            Self::SuperInstance(p) => p.to_agal_boolean(stack, env),
        }
    }

    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        match self {
            Self::Array(a) => Ok(a.clone()),
            Self::Class(c) => c.to_agal_array(stack),
            Self::Error(e) => e.to_agal_array(stack),
            Self::Function(f) => f.to_agal_array(stack),
            Self::Object(o) => o.to_agal_array(stack),
            Self::SuperInstance(s) => s.to_agal_array(stack),
        }
    }

    fn to_agal_byte(self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_byte(stack),
            Self::Class(c) => c.to_agal_byte(stack),
            Self::Error(e) => e.to_agal_byte(stack),
            Self::Function(f) => f.to_agal_byte(stack),
            Self::Object(o) => o.to_agal_byte(stack),
            Self::SuperInstance(s) => s.to_agal_byte(stack),
        }
    }

    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_value(stack, env),
            Self::Class(c) => c.to_agal_value(stack, env),
            Self::Error(e) => e.to_agal_value(stack, env),
            Self::Function(f) => f.to_agal_value(stack, env),
            Self::Object(o) => o.to_agal_value(stack, env),
            Self::SuperInstance(s) => s.to_agal_value(stack, env),
        }
    }

    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::Array(a) => a.to_agal_console(stack, env),
            Self::Class(c) => c.to_agal_console(stack, env),
            Self::Error(e) => e.to_agal_console(stack, env),
            Self::Function(f) => f.to_agal_console(stack, env),
            Self::Object(o) => o.to_agal_console(stack, env),
            Self::SuperInstance(s) => s.to_agal_console(stack, env),
        }
    }

    fn binary_operation(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        match self {
            Self::Array(a) => a.binary_operation(stack, env, operator, other),
            Self::Class(c) => c.binary_operation(stack, env, operator, other),
            Self::Error(e) => e.binary_operation(stack, env, operator, other),
            Self::Function(f) => f.binary_operation(stack, env, operator, other),
            Self::Object(o) => o.binary_operation(stack, env, operator, other),
            Self::SuperInstance(s) => s.binary_operation(stack, env, operator, other),
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
        }
    }

    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        match self {
            Self::Array(a) => a.get_object_property(stack, env, key),
            Self::Class(c) => c.get_object_property(stack, env, key),
            Self::Error(e) => e.get_object_property(stack, env, key),
            Self::Function(f) => f.get_object_property(stack, env, key),
            Self::Object(o) => o.get_object_property(stack, env, key),
            Self::SuperInstance(s) => s.get_object_property(stack, env, key),
        }
    }

    fn set_object_property(
        self,
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
        }
    }

    fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String) {
        match self {
            Self::Array(a) => a.delete_object_property(stack, env, key),
            Self::Class(c) => c.delete_object_property(stack, env, key),
            Self::Error(e) => e.delete_object_property(stack, env, key),
            Self::Function(f) => f.delete_object_property(stack, env, key),
            Self::Object(o) => o.delete_object_property(stack, env, key),
            Self::SuperInstance(s) => s.delete_object_property(stack, env, key),
        }
    }

    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> super::RefAgalValue {
        match self {
            Self::Array(a) => a.get_instance_property(stack, env, key),
            Self::Class(c) => c.get_instance_property(stack, env, key),
            Self::Error(e) => e.get_instance_property(stack, env, key),
            Self::Function(f) => f.get_instance_property(stack, env, key),
            Self::Object(o) => o.get_instance_property(stack, env, key),
            Self::SuperInstance(s) => s.get_instance_property(stack, env, key),
        }
    }

    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        args: Vec<RefAgalValue>,
        modules: &Modules,
    ) -> RefAgalValue {
        match self {
            Self::Array(a) => a.call(stack, env, this, args, modules),
            Self::Class(c) => c.call(stack, env, this, args, modules),
            Self::Error(e) => e.call(stack, env, this, args, modules),
            Self::Function(f) => f.call(stack, env, this, args, modules),
            Self::Object(o) => o.call(stack, env, this, args, modules),
            Self::SuperInstance(s) => s.call(stack, env, this, args, modules),
        }
    }
}
