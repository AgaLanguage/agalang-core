use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
    runtime::{
        env::{RefEnvironment, FALSE_KEYWORD, NOTHING_KEYWORD, NULL_KEYWORD, TRUE_KEYWORD},
        Stack,
    },
    Modules,
};
use parser::{ast::Node, internal::ErrorNames};

pub mod primitive;
pub use primitive::*;
pub mod complex;
pub use complex::*;
pub mod internal;
pub use internal::*;

pub type RefAgalValue = Rc<RefCell<AgalValue>>;

#[derive(Clone)]
pub enum AgalValue {
    Internal(AgalInternal),
    Primitive(AgalPrimitive),
    Complex(AgalComplex),
    Export(String, RefAgalValue),
    Return(RefAgalValue),
    Continue,
    Never,
    Break,
    Null,
}
impl PartialEq for AgalValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AgalValue::Internal(a), AgalValue::Internal(b)) => a == b,
            (AgalValue::Primitive(a), AgalValue::Primitive(b)) => a == b,
            (AgalValue::Complex(a), AgalValue::Complex(b)) => a == b,
            (AgalValue::Export(name, value), AgalValue::Export(on, ov)) => {
                name == on && value == ov
            }
            (AgalValue::Never, AgalValue::Never) => true,
            (AgalValue::Null, AgalValue::Null) => true,
            (_, _) => false,
        }
    }
}
impl AgalValue {
    pub fn as_ref(self) -> RefAgalValue {
        Rc::new(RefCell::new(self))
    }
    pub fn is_never(&self) -> bool {
        match self {
            AgalValue::Never => true,
            _ => false,
        }
    }
    pub fn is_stop(&self) -> bool {
        match self {
            AgalValue::Break
            | AgalValue::Continue
            | AgalValue::Return(_)
            | AgalValue::Internal(AgalInternal::Throw(_)) => true,
            _ => false,
        }
    }
    pub fn is_break(&self) -> bool {
        match self {
            AgalValue::Break => true,
            _ => false,
        }
    }
    pub fn is_return(&self) -> bool {
        match self {
            AgalValue::Return(_) => true,
            _ => false,
        }
    }
    pub fn is_throw(&self) -> bool {
        match self {
            AgalValue::Internal(AgalInternal::Throw(_)) => true,
            _ => false,
        }
    }
    pub fn get_throw(&self) -> Option<AgalThrow> {
        match self {
            AgalValue::Internal(AgalInternal::Throw(t)) => Some(t.clone()),
            _ => None,
        }
    }
    pub fn is_export(&self) -> bool {
        match self {
            AgalValue::Export(_, _) => true,
            _ => false,
        }
    }
    pub fn get_export(&self) -> Option<(String, RefAgalValue)> {
        match self {
            AgalValue::Export(name, value) => Some((name.clone(), value.clone())),
            _ => None,
        }
    }
}
impl AgalValuableManager for AgalValue {
    fn to_value(self) -> AgalValue {
        self
    }
    fn to_agal_number(self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        match self {
            AgalValue::Internal(i) => i.to_agal_number(stack, env),
            AgalValue::Primitive(p) => p.to_agal_number(stack, env),
            AgalValue::Complex(c) => c.to_agal_number(stack, env),
            AgalValue::Null => Ok(AgalNumber::new(0f64)),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en numero".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Null => Ok(AgalString::from_string(NULL_KEYWORD.to_string())),
            AgalValue::Never => Ok(AgalString::from_string(NOTHING_KEYWORD.to_string())),
            AgalValue::Internal(i) => i.to_agal_string(stack, env),
            AgalValue::Complex(c) => c.to_agal_string(stack, env),
            AgalValue::Primitive(p) => p.to_agal_string(stack, env),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en cadena".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        match self {
            AgalValue::Null => env
                .as_ref()
                .borrow()
                .get(stack, FALSE_KEYWORD, &Node::None)
                .as_ref()
                .borrow()
                .clone()
                .to_agal_boolean(stack, env.clone()),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
            AgalValue::Internal(i) => i.to_agal_boolean(stack, env),
            AgalValue::Complex(c) => c.to_agal_boolean(stack, env),
            AgalValue::Primitive(p) => p.to_agal_boolean(stack, env),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        match self {
            AgalValue::Internal(i) => i.to_agal_array(stack),
            AgalValue::Complex(c) => c.to_agal_array(stack),
            AgalValue::Primitive(p) => p.to_agal_array(stack),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Iterable"),
                message: "El valor no es iterable".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_byte(self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        match self {
            AgalValue::Internal(i) => i.to_agal_byte(stack),
            AgalValue::Complex(c) => c.to_agal_byte(stack),
            AgalValue::Primitive(p) => p.to_agal_byte(stack),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Iterable"),
                message: "El valor no es iterable".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Internal(i) => i.to_agal_console(stack, env),
            AgalValue::Complex(c) => c.to_agal_console(stack, env),
            AgalValue::Primitive(p) => p.to_agal_console(stack, env),
            AgalValue::Null => Ok(AgalString::from_string("\x1b[97mnulo\x1b[39m".to_string())),
            _ => Ok(AgalString::from_string("\x1b[90mnada\x1b[39m".to_string())),
        }
    }
    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Internal(i) => i.to_agal_value(stack, env),
            AgalValue::Complex(c) => c.to_agal_value(stack, env),
            AgalValue::Primitive(p) => p.to_agal_value(stack, env),
            _ => self.to_agal_console(stack, env),
        }
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        match self {
            AgalValue::Internal(i) => i.get_instance_property(stack, env, key),
            AgalValue::Complex(c) => c.get_instance_property(stack, env, key),
            AgalValue::Primitive(p) => p.get_instance_property(stack, env, key),
            AgalValue::Return(r) => r
                .as_ref()
                .borrow()
                .clone()
                .get_instance_property(stack, env, key),
            AgalValue::Never | AgalValue::Null => AgalError::new(
                ErrorNames::CustomError("Error Propiedad"),
                format!("No se puede obtener la propiedad {}", key),
                Box::new(stack.clone()),
            )
            .to_ref_value(),
            _ => self.as_ref(),
        }
    }
    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        match self {
            AgalValue::Internal(i) => i.get_object_property(stack, env, key),
            AgalValue::Complex(c) => c.get_object_property(stack, env, key),
            AgalValue::Primitive(p) => p.get_object_property(stack, env, key),
            _ => get_property_error(stack, env, key),
        }
    }
    fn set_object_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        let data = self;
        match data {
            AgalValue::Internal(i) => i.set_object_property(stack, env, key, value),
            AgalValue::Complex(c) => c.set_object_property(stack, env, key, value),
            AgalValue::Primitive(p) => p.set_object_property(stack, env, key, value),
            _ => AgalValue::Never.as_ref(),
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
            AgalValue::Internal(i) => i.binary_operation(stack, env, operator, other),
            AgalValue::Complex(c) => c.binary_operation(stack, env, operator, other),
            AgalValue::Primitive(p) => p.binary_operation(stack, env, operator, other),
            _ => {
                let a: &AgalValue = self.borrow();
                let b: &AgalValue = &other.as_ref().borrow();
                AgalBoolean::new(a == b).to_ref_value()
            }
        }
    }
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match self {
            AgalValue::Internal(i) => i.unary_operator(stack, env, operator),
            AgalValue::Complex(c) => c.unary_operator(stack, env, operator),
            AgalValue::Primitive(p) => p.unary_operator(stack, env, operator),
            _ => AgalValue::Never.as_ref(),
        }
    }
    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match self {
            AgalValue::Internal(i) => i.unary_back_operator(stack, env, operator),
            AgalValue::Complex(c) => c.unary_back_operator(stack, env, operator),
            AgalValue::Primitive(p) => p.unary_back_operator(stack, env, operator),
            _ => AgalValue::Never.as_ref(),
        }
    }
    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        args: Vec<RefAgalValue>,
        modules_manager: &Modules,
    ) -> RefAgalValue {
        match self {
            AgalValue::Internal(n) => n.call(stack, env, this, args, modules_manager),
            AgalValue::Complex(c) => c.call(stack, env, this, args, modules_manager),
            AgalValue::Primitive(p) => p.call(stack, env, this, args, modules_manager),
            _ => {
                let message = format!("No se puede llamar a {}", self.get_type());
                AgalThrow::Params {
                    type_error: ErrorNames::CustomError("Error Llamada"),
                    message,
                    stack: Box::new(stack.clone()),
                }
                .to_ref_value()
            }
        }
    }

    fn get_type(self) -> &'static str {
        match self {
            AgalValue::Internal(i) => i.get_type(),
            AgalValue::Complex(c) => c.get_type(),
            AgalValue::Primitive(p) => p.get_type(),
            AgalValue::Null => "Nulo",
            _ => "Nada",
        }
    }

    fn get_keys(self) -> Vec<String> {
        match self {
            AgalValue::Internal(i) => i.get_keys(),
            AgalValue::Complex(c) => c.get_keys(),
            AgalValue::Primitive(p) => p.get_keys(),
            _ => vec![],
        }
    }

    fn get_length(self) -> usize {
        match self {
            AgalValue::Internal(i) => i.get_length(),
            AgalValue::Complex(c) => c.get_length(),
            AgalValue::Primitive(p) => p.get_length(),
            _ => 0,
        }
    }

    fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String) {
        match self {
            AgalValue::Internal(i) => i.delete_object_property(stack, env, key),
            AgalValue::Complex(c) => c.delete_object_property(stack, env, key),
            AgalValue::Primitive(p) => p.delete_object_property(stack, env, key),
            _ => (),
        }
    }
}

pub trait AgalValuableManager
where
    Self: Sized + Clone + PartialEq,
{
    fn get_type(self) -> &'static str;
    fn to_value(self) -> AgalValue;
    fn to_ref_value(self) -> RefAgalValue {
        self.to_value().as_ref()
    }
    fn get_keys(self) -> Vec<String>;
    fn get_length(self) -> usize;
    // types
    fn to_agal_number(self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow>;
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow>;
    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow>;
    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow>;
    fn to_agal_byte(self, stack: &Stack) -> Result<AgalByte, AgalThrow>;

    // utils
    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow>;
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow>;

    fn binary_operation(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue;

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue;

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue;
    // object methods
    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue;
    fn set_object_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue;
    fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String);
    // instance methods
    fn get_instance_property(self, stack: &Stack, env: RefEnvironment, key: String)
        -> RefAgalValue;

    // values
    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        args: Vec<RefAgalValue>,
        modules: &Modules,
    ) -> RefAgalValue;
}

pub trait AgalValuable
where
    Self: Sized + Clone,
{
    fn to_value(self) -> AgalValue;
    fn to_ref_value(self) -> RefAgalValue {
        self.to_value().as_ref()
    }
    fn get_keys(self) -> Vec<String> {
        vec![]
    }
    fn get_length(self) -> usize {
        0
    }
    // types
    fn to_agal_number(self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo"),
            message: "No se pudo convertir en numero".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<interno>".to_string()))
    }
    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        let value = env
            .as_ref()
            .borrow()
            .get(stack, TRUE_KEYWORD, stack.get_value());
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
    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Iterable"),
            message: "El valor no es iterable".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_byte(self, stack: &Stack) -> Result<AgalByte, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::TypeError,
            message: "El valor no es un byte".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    // utils
    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow>;

    fn binary_operation(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        binary_operation_error(stack, operator, self.clone().to_ref_value(), other)
    }

    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        unary_operation_error(stack, operator, self.clone().to_ref_value())
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        unary_back_operation_error(stack, operator, self.clone().to_ref_value())
    }
    // object methods
    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        get_property_error(stack, env, key)
    }
    fn set_object_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        set_property_error(stack, env, key, "No se puede asignar".to_string())
    }
    fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String) {
        delete_property_error(stack, env, key);
    }
    // instance methods
    fn get_instance_property(self, stack: &Stack, env: RefEnvironment, key: String)
        -> RefAgalValue;

    // values
    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        args: Vec<RefAgalValue>,
        modules: &Modules,
    ) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
}

pub fn set_property_error(
    stack: &Stack,
    env: RefEnvironment,
    key: String,
    message: String,
) -> RefAgalValue {
    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede obtener la propiedad {}: {}", key, message),
        stack: Box::new(stack.clone()),
    }
    .to_ref_value()
}
pub fn get_property_error(stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede obtener la propiedad {}", key),
        stack: Box::new(stack.clone()),
    }
    .to_ref_value()
}
pub fn delete_property_error(stack: &Stack, env: RefEnvironment, key: String) -> AgalValue {
    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede eliminar la propiedad {}", key),
        stack: Box::new(stack.clone()),
    }
    .to_value()
}

pub fn get_instance_property_error(
    stack: &Stack,
    env: RefEnvironment,
    key: String,
    value: AgalValue,
) -> RefAgalValue {
    let rc_stack: Rc<RefCell<Stack>> = Rc::new(RefCell::new(stack.clone()));
    match key.as_str() {
        "aCadena" => {
            let key_clone = key.clone();
            crate::runtime::AgalNativeFunction {
                name: "aCadena".to_string(),
                func: Rc::new({
                    let e_stack = Rc::clone(&rc_stack);
                    let e_env = Rc::clone(&env);
                    move |_, _, _, _, _| -> RefAgalValue {
                        let data = get_instance_property_value(
                            e_stack.clone(),
                            e_env.clone(),
                            &key_clone,
                            &value,
                        );
                        data
                    }
                }),
            }
            .to_ref_value()
        }
        "__aConsola__" => {
            let key_clone = key.clone();
            crate::runtime::AgalNativeFunction {
                name: "__aConsola__".to_string(),
                func: Rc::new({
                    let e_stack = Rc::clone(&rc_stack);
                    let e_env = Rc::clone(&env);
                    move |_, _, _, _, _| -> RefAgalValue {
                        let data = get_instance_property_value(
                            e_stack.clone(),
                            e_env.clone(),
                            &key_clone,
                            &value,
                        );
                        data
                    }
                }),
            }
            .to_ref_value()
        }
        "aNumero" => {
            let key_clone = key.clone();
            crate::runtime::AgalNativeFunction {
                name: "aNumero".to_string(),
                func: Rc::new({
                    let e_stack = Rc::clone(&rc_stack);
                    let e_env = Rc::clone(&env);
                    move |_, _, _, _, _| -> RefAgalValue {
                        let data = get_instance_property_value(
                            e_stack.clone(),
                            e_env.clone(),
                            &key_clone,
                            &value,
                        );
                        data
                    }
                }),
            }
            .to_ref_value()
        }
        "aBuleano" => {
            let key_clone = key.clone();
            crate::runtime::AgalNativeFunction {
                name: "aBuleano".to_string(),
                func: Rc::new({
                    let e_stack = Rc::clone(&rc_stack);
                    let e_env = Rc::clone(&env);
                    move |_, _, _, _, _| -> RefAgalValue {
                        let data = get_instance_property_value(
                            e_stack.clone(),
                            e_env.clone(),
                            &key_clone,
                            &value,
                        );
                        data
                    }
                }),
            }
            .to_ref_value()
        }
        "__aIterable__" => {
            let key_clone = key.clone();
            crate::runtime::AgalNativeFunction {
                name: "__aIterable__".to_string(),
                func: Rc::new({
                    let e_stack = Rc::clone(&rc_stack);
                    let e_env = Rc::clone(&env);
                    move |_, _, _, _, _| -> RefAgalValue {
                        let data = get_instance_property_value(
                            e_stack.clone(),
                            e_env.clone(),
                            &key_clone,
                            &value,
                        );
                        data
                    }
                }),
            }
            .to_ref_value()
        }
        _ => get_property_error(stack, env, key),
    }
}

fn get_instance_property_value(
    stack: Rc<RefCell<Stack>>,
    env: RefEnvironment,
    key: &str,
    value: &AgalValue,
) -> RefAgalValue {
    let stack = &stack.as_ref().borrow();
    match key {
        "aCadena" => {
            let data = value.clone().to_agal_string(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            match data {
                Ok(s) => s.to_ref_value(),
                Err(e) => e.to_ref_value(),
            }
        }
        "__aConsola__" => {
            let data = value.clone().to_agal_console(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            match data {
                Ok(s) => s.to_ref_value(),
                Err(e) => e.to_ref_value(),
            }
        }
        "aNumero" => {
            let data = value.clone().to_agal_number(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            match data {
                Ok(s) => s.to_ref_value(),
                Err(e) => e.to_ref_value(),
            }
        }
        "aBuleano" => {
            let data = value.clone().to_agal_boolean(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            match data {
                Ok(s) => s.to_ref_value(),
                Err(e) => e.to_ref_value(),
            }
        }
        "__aIterable__" => {
            let data = value.clone().to_agal_array(stack);
            match data {
                Ok(s) => s.to_ref_value(),
                Err(e) => e.to_ref_value(),
            }
        }
        _ => get_property_error(stack, env, key.to_string()),
    }
}
pub fn binary_operation_error(
    stack: &Stack,
    operator: &str,
    left: RefAgalValue,
    right: RefAgalValue,
) -> RefAgalValue {
    let left = left.as_ref().borrow();
    let right = right.as_ref().borrow();

    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {} {}",
            left.clone().get_type(),
            operator,
            right.clone().get_type()
        ),
        stack: Box::new(stack.clone()),
    }
    .to_ref_value()
}
pub fn unary_operation_error(stack: &Stack, operator: &str, value: RefAgalValue) -> RefAgalValue {
    let value = value.as_ref().borrow();

    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {}",
            operator,
            value.clone().get_type(),
        ),
        stack: Box::new(stack.clone()),
    }
    .to_ref_value()
}
pub fn unary_back_operation_error(
    stack: &Stack,
    operator: &str,
    value: RefAgalValue,
) -> RefAgalValue {
    let value: std::cell::Ref<'_, AgalValue> = value.as_ref().borrow();

    if value.is_throw() && operator == "?" {
        return AgalValue::Null.as_ref();
    }

    AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {}",
            value.clone().get_type(),
            operator,
        ),
        stack: Box::new(stack.clone()),
    }
    .to_ref_value()
}
