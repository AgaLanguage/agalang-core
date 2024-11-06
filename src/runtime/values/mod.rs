use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use super::{
    env::{RefEnvironment, FALSE_KEYWORD, NOTHING_KEYWORD, NULL_KEYWORD, TRUE_KEYWORD},
    Stack,
};
use parser::{ast::Node, internal::ErrorNames};

pub mod primitive;
pub use primitive::*;
pub mod complex;
pub use complex::*;
pub mod internal;
pub use internal::*;

pub type RefAgalValue = Rc<RefCell<AgalValue>>;

pub enum AgalValue {
    NativeFunction(AgalNativeFunction),
    Export(String, RefAgalValue),
    Function(AgalFunction),
    Boolean(AgalBoolean),
    Return(RefAgalValue),
    Object(AgalObject),
    Number(AgalNumber),
    String(AgalString),
    Error(AgalError),
    Throw(AgalThrow),
    Array(AgalArray),
    Class(AgalClass),
    Char(AgalChar),
    Byte(AgalByte),
    Continue,
    Never,
    Break,
    Null,
}
impl Clone for AgalValue {
    fn clone(&self) -> Self {
        match self {
            AgalValue::Class(c) => c.clone().to_value(),
            AgalValue::Byte(b) => b.clone().to_value(),
            AgalValue::NativeFunction(f) => AgalValue::NativeFunction(f.clone()),
            AgalValue::Number(n) => n.clone().to_value(),
            AgalValue::String(s) => s.clone().to_value(),
            AgalValue::Boolean(b) => b.clone().to_value(),
            AgalValue::Char(c) => c.clone().to_value(),
            AgalValue::Null => AgalValue::Null,
            AgalValue::Never => AgalValue::Never,
            AgalValue::Continue => AgalValue::Continue,
            AgalValue::Break => AgalValue::Break,
            AgalValue::Error(e) => e.clone().to_value(),
            AgalValue::Throw(t) => t.clone().to_value(),
            AgalValue::Function(f) => f.clone().to_value(),
            AgalValue::Array(a) => a.clone().to_value(),
            AgalValue::Object(o) => o.clone().to_value(),
            AgalValue::Return(r) => AgalValue::Return(r.clone()),
            AgalValue::Export(name, value) => AgalValue::Export(name.clone(), value.clone()),
        }
    }
}
impl PartialEq for AgalValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AgalValue::Class(c), AgalValue::Class(o)) => c == o,
            (AgalValue::Byte(b), AgalValue::Byte(o)) => b == o,
            (AgalValue::NativeFunction(f), AgalValue::NativeFunction(o)) => {
                f as *const _ == o as *const _
            }
            (AgalValue::Number(n), AgalValue::Number(o)) => n == o,
            (AgalValue::String(s), AgalValue::String(o)) => s == o,
            (AgalValue::Boolean(b), AgalValue::Boolean(o)) => b == o,
            (AgalValue::Char(c), AgalValue::Char(o)) => c == o,
            (AgalValue::Null, AgalValue::Null) => true,
            (AgalValue::Error(e), AgalValue::Error(o)) => e == o,
            (AgalValue::Throw(t), AgalValue::Throw(o)) => t == o,
            (AgalValue::Function(f), AgalValue::Function(o)) => f == o,
            (AgalValue::Array(a), AgalValue::Array(o)) => a == o,
            (AgalValue::Object(o), AgalValue::Object(oo)) => o == oo,
            (AgalValue::Return(r), AgalValue::Return(o)) => r == o,
            (AgalValue::Export(name, value), AgalValue::Export(on, ov)) => {
                name == on && value == ov
            }
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
            AgalValue::Break | AgalValue::Continue | AgalValue::Return(_) | AgalValue::Throw(_) => {
                true
            }
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
            AgalValue::Throw(_) => true,
            _ => false,
        }
    }
    pub fn get_throw(&self) -> Option<AgalThrow> {
        match self {
            AgalValue::Throw(t) => Some(t.clone()),
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
    pub fn get_type(&self) -> &str {
        match self {
            AgalValue::Class(_) => "Clase",
            AgalValue::Error(_) => "Error",
            AgalValue::Number(_) => "Numero",
            AgalValue::String(_) => "Cadena",
            AgalValue::Boolean(_) => "Booleano",
            AgalValue::Char(_) => "Caracter",
            AgalValue::Null => "Nulo",
            AgalValue::Never => "Nada",
            AgalValue::Break => "Romper",
            AgalValue::Continue => "Continuar",
            AgalValue::Throw(_) => "Error",
            AgalValue::Function(_) => "Funcion",
            AgalValue::Array(_) => "Arreglo",
            AgalValue::Object(_) => "Objeto",
            AgalValue::NativeFunction(_) => "Funcion nativa",
            AgalValue::Return(_) => "Retorno",
            AgalValue::Byte(_) => "Byte",
            AgalValue::Export(_, _) => "Export",
        }
    }
}
impl AgalValuable for AgalValue {
    fn to_value(self) -> AgalValue {
        self
    }
    fn to_agal_number(self, stack: &Stack, env: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        match self {
            AgalValue::Number(n) => Ok(n),
            AgalValue::String(s) => s.to_agal_number(stack, env),
            AgalValue::Boolean(b) => b.to_agal_number(stack, env),
            AgalValue::Null => Ok(AgalNumber::new(0f64)),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en numero".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Number(n) => n.to_agal_string(stack, env),
            AgalValue::String(s) => Ok(s),
            AgalValue::Boolean(b) => b.to_agal_string(stack, env),
            AgalValue::Null => Ok(AgalString::from_string(NULL_KEYWORD.to_string())),
            AgalValue::Never => Ok(AgalString::from_string(NOTHING_KEYWORD.to_string())),
            AgalValue::Throw(t) => Err(t),
            AgalValue::Array(a) => a.to_agal_string(stack, env),
            AgalValue::Char(c) => c.to_agal_string(stack, env),
            AgalValue::Byte(b) => b.to_agal_string(stack, env),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en cadena".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        match self {
            AgalValue::Number(n) => n.to_agal_boolean(stack, env),
            AgalValue::String(s) => s.to_agal_boolean(stack, env),
            AgalValue::Boolean(b) => Ok(b),
            AgalValue::Null => env
                .as_ref()
                .borrow()
                .get(FALSE_KEYWORD, &Node::None)
                .as_ref()
                .borrow()
                .clone()
                .to_agal_boolean(stack, env.clone()),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo"),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        match self {
            AgalValue::Array(a) => Ok(a),
            AgalValue::String(a) => a.to_agal_array(stack),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Iterable"),
                message: "El valor no es iterable".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Array(a)=>a.to_agal_console(stack, env),
            AgalValue::Boolean(b)=>b.to_agal_console(stack, env),
            AgalValue::Byte(b)=>b.to_agal_console(stack, env),
            AgalValue::Char(c)=>c.to_agal_console(stack, env),
            AgalValue::Class(c)=>c.to_agal_console(stack, env),
            AgalValue::Error(e)=>e.to_agal_console(stack, env),
            AgalValue::Number(n)=>n.to_agal_console(stack, env),
            AgalValue::Object(o)=>o.to_agal_console(stack, env),
            AgalValue::Function(f)=>f.to_agal_console(stack, env),
            AgalValue::NativeFunction(n)=>n.to_agal_console(stack, env),
            AgalValue::String(s)=>s.to_agal_console(stack, env),
            AgalValue::Null=>Ok(AgalString::from_string("\x1b[97mnulo\x1b[39m".to_string())),
            _=>Ok(AgalString::from_string("\x1b[90mnada\x1b[39m".to_string())),
        }
    }    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::String(s)=>s.to_agal_value(stack, env),
            _=>self.to_agal_console(stack, env),
        }
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        match self {
            AgalValue::Array(a) => a.get_instance_property(stack, env, key),
            AgalValue::Boolean(b) => b.get_instance_property(stack, env, key),
            AgalValue::Char(c) => c.get_instance_property(stack, env, key),
            AgalValue::Error(e) => e.get_instance_property(stack, env, key),
            AgalValue::Byte(b) => b.get_instance_property(stack, env, key),
            AgalValue::Number(n) => n.get_instance_property(stack, env, key),
            AgalValue::String(s) => s.get_instance_property(stack, env, key),
            AgalValue::Function(f) => f.get_instance_property(stack, env, key),
            AgalValue::NativeFunction(n) => {
                n.borrow().clone().get_instance_property(stack, env, key)
            }
            AgalValue::Return(r) => r
                .as_ref()
                .borrow()
                .clone()
                .get_instance_property(stack, env, key),
            AgalValue::Object(o) => o.get_instance_property(stack, env, key),
            AgalValue::Never | AgalValue::Null => AgalValue::Error(AgalError::new(
                ErrorNames::CustomError("Error Propiedad"),
                format!("No se puede obtener la propiedad {}", key),
                Box::new(stack.clone()),
            ))
            .as_ref(),
            _ => self.as_ref(),
        }
    }
    fn get_object_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        match self {
            AgalValue::Object(o) => o.get_object_property(stack, env, key),
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
            AgalValue::Object(o) => o.set_object_property(stack, env, key, value),
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
            AgalValue::Number(n) => n.binary_operation(stack, env, operator, other),
            AgalValue::String(s) => s.binary_operation(stack, env, operator, other),
            AgalValue::Boolean(b) => b.binary_operation(stack, env, operator, other),
            AgalValue::Char(c) => c.binary_operation(stack, env, operator, other),
            AgalValue::Byte(b) => b.binary_operation(stack, env, operator, other),
            AgalValue::Array(a) => a.binary_operation(stack, env, operator, other),
            AgalValue::Object(o) => o.binary_operation(stack, env, operator, other),
            AgalValue::Throw(t) => AgalValue::Throw(t.clone()).as_ref(),
            _ => AgalValue::Never.as_ref(),
        }
    }
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match self {
            AgalValue::Number(n) => n.unary_operator(stack, env, operator),
            AgalValue::String(s) => s.unary_operator(stack, env, operator),
            AgalValue::Boolean(b) => b.unary_operator(stack, env, operator),
            AgalValue::Char(c) => c.unary_operator(stack, env, operator),
            AgalValue::Byte(b) => b.unary_operator(stack, env, operator),
            AgalValue::Array(a) => a.unary_operator(stack, env, operator),
            AgalValue::Object(o) => o.unary_operator(stack, env, operator),
            AgalValue::Throw(t) => AgalValue::Throw(t.clone()).as_ref(),
            _ => AgalValue::Never.as_ref(),
        }
    }
    fn unary_back_operator(
        &self,
        _stack: &Stack,
        _env: RefEnvironment,
        operator: &str,
    ) -> RefAgalValue {
        match (self, operator) {
            (AgalValue::Throw(_), "?") => AgalValue::Null.as_ref(),
            _ => self.clone().as_ref(),
        }
    }
    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefAgalValue,
        arguments: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        match self {
            AgalValue::Throw(t) => t.call(stack, env, this, arguments),
            AgalValue::Number(n) => n.call(stack, env, this, arguments),
            AgalValue::Object(o) => o.call(stack, env, this, arguments),
            AgalValue::Function(f) => f.call(stack, env, this, arguments),
            AgalValue::NativeFunction(f) => {
                let v = (f.func)(arguments, stack, env);
                v
            }
            AgalValue::Class(c) => c.call(stack, env, this, arguments),
            _ => {
                let message = format!("No se puede llamar a {}", self.get_type());
                AgalValue::Throw(AgalThrow::Params {
                    type_error: ErrorNames::CustomError("Error Llamada"),
                    message,
                    stack: Box::new(stack.clone()),
                })
                .as_ref()
            }
        }
    }
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
    fn to_agal_number(self, stack: &Stack, _: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo"),
            message: "No se pudo convertir en numero".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<interno>".to_string()))
    }
    fn to_agal_boolean(self, stack: &Stack, env: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        let value = env.as_ref().borrow().get(TRUE_KEYWORD, stack.get_value());
        let value: &AgalValue = &value.as_ref().borrow();
        match value {
            AgalValue::Boolean(b) => Ok(b.clone()),
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

    // utils
    fn to_agal_value(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow>;

    fn binary_operation(
        &self,
        stack: &Stack,
        _env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        binary_operation_error(stack, operator, self.clone().to_ref_value(), other)
    }

    fn unary_operator(&self, stack: &Stack, _env: RefEnvironment, operator: &str) -> RefAgalValue {
        unary_operation_error(stack, operator, self.clone().to_ref_value())
    }

    fn unary_back_operator(
        &self,
        stack: &Stack,
        _env: RefEnvironment,
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
        _value: RefAgalValue,
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
        _: &Stack,
        _: RefEnvironment,
        _: RefAgalValue,
        _: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
    fn construct(self, _: &Stack, _: RefEnvironment, _: Vec<RefAgalValue>) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
}

pub fn set_property_error(
    stack: &Stack,
    _env: RefEnvironment,
    key: String,
    message: String,
) -> RefAgalValue {
    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede obtener la propiedad {}: {}", key, message),
        stack: Box::new(stack.clone()),
    })
    .as_ref()
}
pub fn get_property_error(stack: &Stack, _env: RefEnvironment, key: String) -> RefAgalValue {
    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede obtener la propiedad {}", key),
        stack: Box::new(stack.clone()),
    })
    .as_ref()
}
pub fn delete_property_error(stack: &Stack, _env: RefEnvironment, key: String) -> AgalValue {
    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad"),
        message: format!("No se puede eliminar la propiedad {}", key),
        stack: Box::new(stack.clone()),
    })
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
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&env);
                move |_: Vec<RefAgalValue>, _:&Stack, _:RefEnvironment| -> RefAgalValue {
                    let data = get_instance_property_value(
                        e_stack.clone(),
                        e_env.clone(),
                        &key_clone,
                        &value,
                    );
                    data
                }
            };
            let func = Rc::new(function);
            let value = AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                name: "aCadena".to_string(),
                func,
            });
            value.as_ref()
        }
        "__aConsola__" => {
            let key_clone = key.clone();
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&env);
                move |_: Vec<RefAgalValue>, _:&Stack, _:RefEnvironment| -> RefAgalValue {
                    let data = get_instance_property_value(
                        e_stack.clone(),
                        e_env.clone(),
                        &key_clone,
                        &value,
                    );
                    data
                }
            };
            let func = Rc::new(function);
            let value = AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                name: "__aConsola__".to_string(),
                func,
            });
            value.as_ref()
        }
        "aNumero" => {
            let key_clone = key.clone();
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&env);
                move |_: Vec<RefAgalValue>, _:&Stack, _:RefEnvironment| -> RefAgalValue {
                    let data = get_instance_property_value(
                        e_stack.clone(),
                        e_env.clone(),
                        &key_clone,
                        &value,
                    );
                    data
                }
            };
            let func = Rc::new(function);
            let value = AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                name: "aNumero".to_string(),
                func,
            });
            value.as_ref()
        }
        "aBuleano" => {
            let key_clone = key.clone();
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&env);
                move |_: Vec<RefAgalValue>, _:&Stack, _:RefEnvironment| -> RefAgalValue {
                    let data = get_instance_property_value(
                        e_stack.clone(),
                        e_env.clone(),
                        &key_clone,
                        &value,
                    );
                    data
                }
            };
            let func = Rc::new(function);
            let value = AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                name: "aBuleano".to_string(),
                func,
            });
            value.as_ref()
        }
        "__aIterable__" => {
            let key_clone = key.clone();
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&env);
                move |_: Vec<RefAgalValue>, _:&Stack, _:RefEnvironment| -> RefAgalValue {
                    let data = get_instance_property_value(
                        e_stack.clone(),
                        e_env.clone(),
                        &key_clone,
                        &value,
                    );
                    data
                }
            };
            let func = Rc::new(function);
            let value = AgalValue::NativeFunction(crate::runtime::AgalNativeFunction {
                name: "__aIterable__".to_string(),
                func,
            });
            value.as_ref()
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
            let data = value
                .clone()
                .to_agal_string(stack, env.as_ref().borrow().clone().crate_child(false).as_ref());
            let data = match data {
                Ok(s) => AgalValue::String(s),
                Err(e) => AgalValue::Throw(e),
            };
            data.as_ref()
        }
        "__aConsola__" => {
            let data = value
                .clone()
                .to_agal_console(stack, env.as_ref().borrow().clone().crate_child(false).as_ref());
            let data = match data {
                Ok(s) => AgalValue::String(s),
                Err(e) => AgalValue::Throw(e),
            };
            data.as_ref()
        }
        "aNumero" => {
            let data = value
                .clone()
                .to_agal_number(stack, env.as_ref().borrow().clone().crate_child(false).as_ref());
            let data = match data {
                Ok(s) => AgalValue::Number(s),
                Err(e) => AgalValue::Throw(e),
            };
            data.as_ref()
        }
        "aBuleano" => {
            let data = value
                .clone()
                .to_agal_boolean(stack, env.as_ref().borrow().clone().crate_child(false).as_ref());
            let data = match data {
                Ok(s) => AgalValue::Boolean(s),
                Err(e) => AgalValue::Throw(e),
            };
            data.as_ref()
        }
        "__aIterable__" => {
            let data = value.clone().to_agal_array(stack);
            let data = match data {
                Ok(s) => AgalValue::Array(s),
                Err(e) => AgalValue::Throw(e),
            };
            data.as_ref()
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

    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {} {}",
            left.get_type(),
            operator,
            right.get_type()
        ),
        stack: Box::new(stack.clone()),
    })
    .as_ref()
}
pub fn unary_operation_error(stack: &Stack, operator: &str, value: RefAgalValue) -> RefAgalValue {
    let value = value.as_ref().borrow();

    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {}",
            operator,
            value.get_type(),
        ),
        stack: Box::new(stack.clone()),
    })
    .as_ref()
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

    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion"),
        message: format!(
            "No se puede realizar la operacion {} {}",
            value.get_type(),
            operator,
        ),
        stack: Box::new(stack.clone()),
    })
    .as_ref()
}
