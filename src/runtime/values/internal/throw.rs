use std::{cell::RefCell, rc::Rc};

use parser::internal::ErrorNames;

use crate::{
    runtime::{
        env::RefEnvironment, AgalError, AgalInternal, AgalPrimitive, AgalString, AgalValuable,
        AgalValuableManager, AgalValue, RefAgalValue, Stack,
    },
    Modules,
};

#[derive(Clone, PartialEq)]
pub enum AgalThrow {
    Params {
        type_error: ErrorNames,
        message: String,
        stack: Box<Stack>,
    },
    Error(AgalError),
}
impl<'a> AgalThrow {
    pub fn get_error(&self) -> AgalError {
        match self {
            AgalThrow::Params {
                type_error,
                message,
                stack,
            } => AgalError::new(type_error.clone(), message.clone(), stack.clone()),
            AgalThrow::Error(e) => e.clone(),
        }
    }
    pub fn from_ref_value<T: AgalValuable<'a>>(
        v: Rc<RefCell<T>>,
        stack: &Stack,
        env: RefEnvironment,
    ) -> AgalThrow {
        let str = v.borrow().to_agal_console(stack, env);
        if str.is_err() {
            return str.err().unwrap();
        }
        let str = str.ok().unwrap();
        let message = str.get_string().clone();
        AgalThrow::Params {
            type_error: ErrorNames::None,
            message,
            stack: Box::new(stack.clone()),
        }
    }
    pub fn from_ref_manager<T: AgalValuableManager<'a>>(
        v: Rc<RefCell<T>>,
        stack: &Stack,
        env: RefEnvironment,
    ) -> AgalThrow {
        let str = v.borrow().to_agal_console(stack, env);
        if str.is_err() {
            return str.err().unwrap();
        }
        let str = str.ok().unwrap();
        let message = str.get_string().clone();
        AgalThrow::Params {
            type_error: ErrorNames::None,
            message,
            stack: Box::new(stack.clone()),
        }
    }
}

impl<'a> AgalValuable<'a> for AgalThrow {
    fn to_value(&self) -> &'a AgalValue {
        &AgalInternal::Throw(self.clone()).to_value()
    }
    async fn call(
        &self,
        _: &Stack,
        _: RefEnvironment,
        _: RefAgalValue<'a>,
        _: Vec<RefAgalValue<'a>>,
        _: &Modules,
    ) -> RefAgalValue {
        self.to_ref_value()
    }
    fn get_instance_property(&self, _: &Stack, _: RefEnvironment, _: String) -> RefAgalValue {
        self.to_ref_value()
    }
    fn to_agal_string(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("<Error>",)))
    }
    fn to_agal_console(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        self.to_agal_string(stack, env)
    }

    fn to_ref_value(&self) -> RefAgalValue {
        self.to_value().as_ref()
    }

    fn get_keys(&self) -> Vec<String> {
        std::vec![]
    }

    fn get_length(&self) -> usize {
        0
    }

    fn to_agal_number(
        &self,
        stack: &Stack,
        env: RefEnvironment,
    ) -> Result<crate::runtime::AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo"),
            message: "No se pudo convertir en numero".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    fn to_agal_boolean(
        &self,
        stack: &Stack,
        env: RefEnvironment,
    ) -> Result<crate::runtime::AgalBoolean, AgalThrow> {
        let value = env.as_ref().borrow();
        let value = value.get(stack, crate::runtime::env::TRUE_KEYWORD, stack.get_value());
        let value: &AgalValue = &value.as_ref().borrow();
        match value {
            &AgalValue::Primitive(AgalPrimitive::Boolean(b)) => Ok(b),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }

    fn to_agal_array(&self, stack: &Stack) -> Result<&crate::runtime::AgalArray, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Iterable"),
            message: "El valor no es iterable".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    fn to_agal_byte(&self, stack: &Stack) -> Result<crate::runtime::AgalByte, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::TypeError,
            message: "El valor no es un byte".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    fn to_agal_value(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }

    fn binary_operation(
        &self,
        stack: &Stack,
        _env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        crate::runtime::binary_operation_error(stack, operator, self.clone().to_ref_value(), other)
    }

    fn unary_operator(&self, stack: &Stack, _env: RefEnvironment, operator: &str) -> RefAgalValue {
        crate::runtime::unary_operation_error(stack, operator, self.clone().to_ref_value())
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        _env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match operator {
            "?" => AgalValue::Null.as_ref(),
            _ => crate::runtime::unary_back_operation_error(
                stack,
                operator,
                self.clone().to_ref_value(),
            ),
        }
    }

    fn get_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        crate::runtime::get_property_error(stack, env, key)
    }

    fn set_object_property(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        _value: RefAgalValue,
    ) -> RefAgalValue {
        crate::runtime::set_property_error(stack, env, key, "No se puede asignar".to_string())
    }

    fn delete_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) {
        crate::runtime::delete_property_error(stack, env, key);
    }
}

impl std::fmt::Display for AgalThrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalThrow::Params {
                type_error,
                message,
                stack,
            } => {
                write!(f, "{}: {}\n{}", type_error, message, stack)
            }
            AgalThrow::Error(_) => write!(f, "Error"),
        }
    }
}
