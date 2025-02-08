mod native_function;
pub use native_function::*;
mod throw;
pub use throw::*;
mod lazy;
pub use lazy::*;

use crate::runtime::{
    env::RefEnvironment, get_instance_property_error, AgalString, AgalValuable,
    AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

#[derive(Clone)]
pub enum AgalInternal {
    NativeFunction(AgalNativeFunction),
    Throw(AgalThrow),
    Lazy(AgalLazy),
}

impl AgalValuableManager for AgalInternal {
    fn get_type(self) -> &'static str {
        match self {
            Self::NativeFunction(_) => "Funcion nativa",
            Self::Throw(_) => "Lanzado",
            Self::Lazy(_) => "Vago",
        }
    }
    fn to_value(self) -> AgalValue {
        AgalValue::Internal(self)
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_console(stack, env),
            Self::Throw(t) => t.to_agal_console(stack, env),
            Self::Lazy(l) => l.to_agal_console(stack, env),
        }
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        match self {
            Self::NativeFunction(n) => n.get_instance_property(stack, env, key),
            Self::Throw(t) => t.get_instance_property(stack, env, key),
            Self::Lazy(l) => l.get_instance_property(stack, env, key),
        }
    }
    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        args: Vec<RefAgalValue>,
        modules: &crate::Modules,
    ) -> RefAgalValue {
        match self {
            Self::NativeFunction(n) => n.call(stack, env, this, args, modules),
            Self::Throw(t) => t.call(stack, env, this, args, modules),
            Self::Lazy(l) => l.call(stack, env, this, args, modules),
        }
    }
    fn to_agal_boolean(
        self,
        stack: &Stack,
        env: RefEnvironment,
    ) -> Result<super::AgalBoolean, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_boolean(stack, env),
            Self::Throw(t) => t.to_agal_boolean(stack, env),
            Self::Lazy(l) => l.to_agal_boolean(stack, env),
        }
    }
    fn to_agal_number(
        self,
        stack: &Stack,
        env: RefEnvironment,
    ) -> Result<super::AgalNumber, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_number(stack, env),
            Self::Throw(t) => t.to_agal_number(stack, env),
            Self::Lazy(l) => l.to_agal_number(stack, env),
        }
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_string(stack, env),
            Self::Throw(t) => t.to_agal_string(stack, env),
            Self::Lazy(l) => l.to_agal_string(stack, env),
        }
    }

    fn to_agal_array(self, stack: &Stack) -> Result<super::AgalArray, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_array(stack),
            Self::Throw(t) => t.to_agal_array(stack),
            Self::Lazy(l) => l.to_agal_array(stack),
        }
    }

    fn to_agal_byte(self, stack: &Stack) -> Result<super::AgalByte, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_byte(stack),
            Self::Throw(t) => t.to_agal_byte(stack),
            Self::Lazy(l) => l.to_agal_byte(stack),
        }
    }

    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            Self::NativeFunction(n) => n.to_agal_value(stack, env),
            Self::Throw(t) => t.to_agal_value(stack, env),
            Self::Lazy(l) => l.to_agal_value(stack, env),
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
            Self::NativeFunction(n) => n.binary_operation(stack, env, operator, other),
            Self::Throw(t) => t.binary_operation(stack, env, operator, other),
            Self::Lazy(l) => l.binary_operation(stack, env, operator, other),
        }
    }

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match self {
            Self::NativeFunction(n) => n.unary_operator(stack, env, operator),
            Self::Throw(t) => t.unary_operator(stack, env, operator),
            Self::Lazy(l) => l.unary_operator(stack, env, operator),
        }
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match self {
            Self::NativeFunction(n) => n.unary_back_operator(stack, env, operator),
            Self::Throw(t) => t.unary_back_operator(stack, env, operator),
            Self::Lazy(l) => l.unary_back_operator(stack, env, operator),
        }
    }

    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        match self {
            Self::NativeFunction(n) => n.get_object_property(stack, env, key),
            Self::Throw(t) => t.get_object_property(stack, env, key),
            Self::Lazy(l) => l.get_object_property(stack, env, key),
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
            Self::NativeFunction(n) => n.set_object_property(stack, env, key, value),
            Self::Throw(t) => t.set_object_property(stack, env, key, value),
            Self::Lazy(l) => l.set_object_property(stack, env, key, value),
        }
    }

    fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String) {
        match self {
            Self::NativeFunction(n) => n.delete_object_property(stack, env, key),
            Self::Throw(t) => t.delete_object_property(stack, env, key),
            Self::Lazy(l) => l.delete_object_property(stack, env, key),
        }
    }

    fn get_keys(self) -> Vec<String> {
        match self {
            Self::NativeFunction(n) => n.get_keys(),
            Self::Throw(t) => t.get_keys(),
            Self::Lazy(l) => l.get_keys(),
        }
    }

    fn get_length(self) -> usize {
        match self {
            Self::NativeFunction(n) => n.get_length(),
            Self::Throw(t) => t.get_length(),
            Self::Lazy(l) => l.get_length(),
        }
    }
}

impl PartialEq for AgalInternal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NativeFunction(a), Self::NativeFunction(b)) => a as *const _ == b as *const _,
            (Self::Throw(a), Self::Throw(b)) => a == b,
            (Self::Lazy(a), Self::Lazy(b)) => a == b,
            _ => false,
        }
    }
}
