use parser::internal::ErrorNames;

use crate::{
    runtime::{
        env::{RefEnvironment, TRUE_KEYWORD},
        Environment, Stack,
    },
    Modules,
};

use super::{
    binary_operation_error, delete_property_error, get_property_error, set_property_error,
    unary_back_operation_error, unary_operation_error, AgalArray, AgalBoolean, AgalByte,
    AgalNumber, AgalPrimitive, AgalString, AgalThrow, AgalValue, RefAgalValue,
};

pub trait AgalValuableManager<'a>
where
    Self: Sized + PartialEq,
{
    fn to_value(self) -> AgalValue<'a>;
    fn to_ref_value(self) -> RefAgalValue<'a> {
        self.to_value().as_ref()
    }
    fn get_type(&self) -> &'static str;
    fn get_keys(&self) -> Vec<String>;
    fn get_length(&self) -> usize;
    // types
    fn to_agal_number(&self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalNumber, AgalThrow>;
    fn to_agal_string(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow>;
    fn to_agal_boolean(&self, stack: &Stack, env: RefEnvironment<'a>)
        -> Result<AgalBoolean, AgalThrow>;
    fn to_agal_array(&self, stack: &Stack) -> Result<&AgalArray<'a>, AgalThrow>;
    fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow>;

    // utils
    fn to_agal_value(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow>;
    fn to_agal_console(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow>;

    fn binary_operation(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        operator: &str,
        other: RefAgalValue<'a>,
    ) -> RefAgalValue<'a>;

    fn unary_operator(&'a self, stack: &Stack, env: RefEnvironment<'a>, operator: &str) -> RefAgalValue;

    fn unary_back_operator(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        operator: &str,
    ) -> RefAgalValue;
    // object methods
    fn get_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) -> RefAgalValue<'a>;
    fn set_object_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue;
    fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String);
    // instance methods
    fn get_instance_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
    ) -> RefAgalValue;

    // values
    async fn call(
        &self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        this: RefAgalValue<'a>,
        args: Vec<RefAgalValue<'a>>,
        modules: &Modules<'a>,
    ) -> RefAgalValue<'a>;
}

pub trait AgalValuable<'a>
where
    Self: Sized,
{
    fn to_value(self) -> AgalValue<'a>;
    fn to_ref_value(self) -> RefAgalValue<'a> {
        self.to_value().as_ref()
    }
    fn get_keys(&self) -> Vec<String> {
        vec![]
    }
    fn get_length(&self) -> usize {
        0
    }
    // types
    fn to_agal_number(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo"),
            message: "No se pudo convertir en numero".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_string(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
        Ok(AgalString::from_string("<interno>"))
    }
    fn to_agal_boolean(
        &self,
        stack: &Stack,
        env: RefEnvironment<'a>,
    ) -> Result<AgalBoolean, AgalThrow> {
        let value_rc = {
            let env = &env.as_ref().borrow();
            env.get(stack, TRUE_KEYWORD, stack.get_value())
        };

        let value_ref = value_rc.as_ref().borrow();

        let value: &AgalValue = &*value_ref;

        match value {
            &AgalValue::Primitive(AgalPrimitive::Boolean(b)) => Ok(b),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }

    fn to_agal_array(&self, stack: &Stack) -> Result<&AgalArray<'a>, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Iterable"),
            message: "El valor no es iterable".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_byte(&self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::TypeError,
            message: "El valor no es un byte".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    // utils
    fn to_agal_value(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow> {
        self.to_agal_console(stack, env)
    }
    fn to_agal_console(&'a self, stack: &Stack, env: RefEnvironment<'a>) -> Result<AgalString<'a>, AgalThrow>;

    fn binary_operation(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue<'a>,
    ) -> RefAgalValue;

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue;

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue;
    // object methods
    fn get_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) -> RefAgalValue<'a> {
        get_property_error(stack, env, key)
    }
    fn set_object_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        set_property_error(stack, env, key, "No se puede asignar".to_string())
    }
    fn delete_object_property(&'a self, stack: &Stack, env: RefEnvironment, key: String) {
        delete_property_error(stack, env, key);
    }
    // instance methods
    fn get_instance_property(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        key: String,
    ) -> RefAgalValue;

    // values
    async fn call(
        &'a self,
        stack: &Stack,
        env: RefEnvironment<'a>,
        this: RefAgalValue<'a>,
        args: Vec<RefAgalValue<'a>>,
        modules: &Modules<'a>,
    ) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
}
