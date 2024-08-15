use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    env::{FALSE_KEYWORD, NOTHING_KEYWORD, NULL_KEYWORD, TRUE_KEYWORD},
    interpreter, Enviroment, Stack,
};
use crate::{
    frontend::ast::{Node, NodeBlock, NodeIdentifier},
    internal::ErrorNames,
};

pub mod primitive;
pub use primitive::*;
pub mod complex;
pub use complex::*;
pub mod internal;
pub use internal::*;
use util::List;

pub type RefAgalValue = Rc<RefCell<AgalValue>>;

pub enum AgalValue {
    NativeFunction(AgalNativeFunction),
    Number(AgalNumber),
    String(AgalString),
    Boolean(AgalBoolean),
    Char(AgalChar),
    Byte(AgalByte),
    Null,
    Never,
    Break,
    Continue,
    Error(AgalError),
    Throw(AgalThrow),
    Function(List<NodeIdentifier>, NodeBlock, Rc<RefCell<Enviroment>>),
    Array(AgalArray),
    Object(AgalObject),
    Return(RefAgalValue),
}
impl Clone for AgalValue {
    fn clone(&self) -> Self {
        match self {
            AgalValue::Byte(b) => AgalValue::Byte(b.clone()),
            AgalValue::NativeFunction(f) => AgalValue::NativeFunction(f.clone()),
            AgalValue::Number(n) => AgalValue::Number(n.clone()),
            AgalValue::String(s) => AgalValue::String(s.clone()),
            AgalValue::Boolean(b) => AgalValue::Boolean(b.clone()),
            AgalValue::Char(c) => AgalValue::Char(c.clone()),
            AgalValue::Null => AgalValue::Null,
            AgalValue::Never => AgalValue::Never,
            AgalValue::Continue => AgalValue::Continue,
            AgalValue::Break => AgalValue::Break,
            AgalValue::Error(e) => AgalValue::Error(e.clone()),
            AgalValue::Throw(t) => AgalValue::Throw(t.clone()),
            AgalValue::Function(params, body, env) => {
                AgalValue::Function(params.clone(), body.clone(), env.clone())
            }
            AgalValue::Array(a) => AgalValue::Array(a.clone()),
            AgalValue::Object(o) => AgalValue::Object(o.clone()),
            AgalValue::Return(r) => AgalValue::Return(r.clone()),
        }
    }
}
impl PartialEq for AgalValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            AgalValue::Byte(b) => match other {
                AgalValue::Byte(o) => b == o,
                _ => false,
            },
            AgalValue::NativeFunction(f) => match other {
                AgalValue::NativeFunction(o) => f as *const _ == o as *const _,
                _ => false,
            },
            AgalValue::Number(n) => match other {
                AgalValue::Number(o) => n == o,
                _ => false,
            },
            AgalValue::String(s) => match other {
                AgalValue::String(o) => s == o,
                _ => false,
            },
            AgalValue::Boolean(b) => match other {
                AgalValue::Boolean(o) => b == o,
                _ => false,
            },
            AgalValue::Char(c) => match other {
                AgalValue::Char(o) => c == o,
                _ => false,
            },
            AgalValue::Null => match other {
                AgalValue::Null => true,
                _ => false,
            },
            AgalValue::Never | AgalValue::Continue | AgalValue::Break => false,
            AgalValue::Error(e) => match other {
                AgalValue::Error(o) => e == o,
                _ => false,
            },
            AgalValue::Throw(t) => match other {
                AgalValue::Throw(o) => t == o,
                _ => false,
            },
            AgalValue::Function(params, body, env) => match other {
                AgalValue::Function(op, ob, oe) => params == op && body == ob && env == oe,
                _ => false,
            },
            AgalValue::Array(a) => match other {
                AgalValue::Array(o) => a == o,
                _ => false,
            },
            AgalValue::Object(o) => match other {
                AgalValue::Object(oo) => o == oo,
                _ => false,
            },
            AgalValue::Return(r) => match other {
                AgalValue::Return(o) => r == o,
                _ => false,
            },
        }
    }
}
impl AgalValue {
    pub fn to_ref(self) -> RefAgalValue {
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
    pub fn is_continue(&self) -> bool {
        match self {
            AgalValue::Continue => true,
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
    // TODO: Remove this code
    pub fn get_type(&self) -> &str {
        match self {
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
            AgalValue::Function(_, _, _) => "Funcion",
            AgalValue::Array(_) => "Arreglo",
            AgalValue::Object(_) => "Objeto",
            AgalValue::NativeFunction(_) => "Funcion nativa",
            AgalValue::Return(_) => "Retorno",
            AgalValue::Byte(_) => "Byte",
        }
    }
}
impl AgalValuable for AgalValue {
    fn to_value(self) -> AgalValue {
        self
    }
    fn to_agal_number(self, stack: &Stack, env: &Enviroment) -> Result<AgalNumber, AgalThrow> {
        match self {
            AgalValue::Number(n) => Ok(n),
            AgalValue::String(s) => s.to_agal_number(stack, env),
            AgalValue::Boolean(b) => b.to_agal_number(stack, env),
            AgalValue::Null => Ok(AgalNumber::new(0f64)),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en numero".to_string(),
                stack: Box::new(stack.clone()),
            }),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en numero".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_string(self, stack: &Stack, env: &Enviroment) -> Result<AgalString, AgalThrow> {
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
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en cadena".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_boolean(self, stack: &Stack, env: &Enviroment) -> Result<AgalBoolean, AgalThrow> {
        match self {
            AgalValue::Number(n) => n.to_agal_boolean(stack, env),
            AgalValue::String(s) => s.to_agal_boolean(stack, env),
            AgalValue::Boolean(b) => Ok(b),
            AgalValue::Null => env
                .get(FALSE_KEYWORD, &Node::None)
                .borrow()
                .clone()
                .to_agal_boolean(stack, env),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
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
                type_error: ErrorNames::CustomError("Error Iterable".to_string()),
                message: "El valor no es iterable".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
        match self {
            AgalValue::Array(a) => a.get_instance_property(stack, env, key),
            AgalValue::Boolean(b) => b.get_instance_property(stack, env, key),
            AgalValue::Char(c) => c.get_instance_property(stack, env, key),
            AgalValue::Error(e) => e.get_instance_property(stack, env, key),
            AgalValue::Byte(b) => b.get_instance_property(stack, env, key),
            AgalValue::Number(n) => n.get_instance_property(stack, env, key),
            AgalValue::String(s) => s.get_instance_property(stack, env, key),
            AgalValue::Function(_, _, _) => AgalValue::Never.to_ref(),
            AgalValue::NativeFunction(n) => n.get_instance_property(stack, env, key),
            AgalValue::Return(r) => r.borrow().clone().get_instance_property(stack, env, key),
            AgalValue::Object(o) => o.get_instance_property(stack, env, key),
            AgalValue::Never | AgalValue::Null => AgalValue::Error(AgalError::new(
                ErrorNames::CustomError("Error Propiedad".to_string()),
                format!("No se puede obtener la propiedad {}", key),
                Box::new(stack.clone()),
            ))
            .to_ref(),
            _ => self.to_ref(),
        }
    }
    fn get_object_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
        match self {
            AgalValue::Object(o) => o.get_object_property(stack, env, key),
            _ => get_property_error(stack, env, key),
        }
    }
    fn set_object_property(
        mut self,
        stack: &Stack,
        env: &Enviroment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        match self {
            AgalValue::Object(o) => o.set_object_property(stack, env, key, value),
            _ => AgalValue::Never.to_ref(),
        }
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        env: &Enviroment,
        operator: String,
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
            AgalValue::Throw(t) => AgalValue::Throw(t.clone()).to_ref(),
            _ => AgalValue::Never.to_ref(),
        }
    }
    fn call(
        self,
        stack: &Stack,
        env: &Enviroment,
        this: RefAgalValue,
        arguments: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        match self {
            AgalValue::Throw(t) => t.call(stack, env, this, arguments),
            AgalValue::Number(n) => n.call(stack, env, this, arguments),
            AgalValue::Object(o) => o.call(stack, env, this, arguments),
            AgalValue::Function(args, body, env) => {
                let mut new_env = env.as_ref().borrow().clone().crate_child();
                for (i, arg) in args.iter().enumerate() {
                    let value = if i < arguments.len() {
                        arguments[i].clone()
                    } else {
                        AgalValue::Never.to_ref()
                    };
                    new_env.define(
                        stack,
                        &arg.name,
                        value,
                        true,
                        &Node::Identifier(arg.clone()),
                    );
                }
                interpreter(&body.to_node(), stack, env)
            }
            AgalValue::NativeFunction(f) => {
                let v = (f.func)(arguments);
                v
            }
            _ => {
                let message = format!("No se puede llamar a {}", self.get_type());
                AgalValue::Throw(AgalThrow::Params {
                    type_error: ErrorNames::CustomError("Error Llamada".to_string()),
                    message,
                    stack: Box::new(stack.clone()),
                })
                .to_ref()
            }
        }
    }
}

impl std::fmt::Display for AgalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalValue::Error(_e) => write!(f, "Error"),
            AgalValue::Char(c) => write!(f, "{}", c.to_char()),
            AgalValue::Number(n) => write!(f, "{}", n.to_f64()),
            AgalValue::String(s) => write!(f, "{}", s.get_string()),
            AgalValue::Boolean(b) => write!(
                f,
                "{}",
                if b.to_bool() {
                    TRUE_KEYWORD
                } else {
                    FALSE_KEYWORD
                }
            ),
            AgalValue::Byte(b) => {
                let data = b.to_u8();
                write!(f, "{data:08b}")
            }
            AgalValue::Null => write!(f, "nulo"),
            AgalValue::Never => write!(f, "nada"),
            AgalValue::Break => write!(f, "romper"),
            AgalValue::Continue => write!(f, "continuar"),
            AgalValue::Throw(t) => write!(f, "{}", t),
            AgalValue::Function(_, _, _) => write!(f, "Funcion"),
            AgalValue::Array(_) => write!(f, "[..]"),
            AgalValue::Object(_) => write!(f, "{{..}}"),
            AgalValue::NativeFunction(_) => write!(f, "Funcion nativa"),
            AgalValue::Return(r) => write!(f, "Retorno: {}", r.borrow()),
        }
    }
}

pub trait AgalValuable
where
    Self: Sized + Clone,
{
    fn to_value(self) -> AgalValue;
    // types
    fn to_agal_number(self, stack: &Stack, _: &Enviroment) -> Result<AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo".to_string()),
            message: "No se pudo convertir en numero".to_string(),
            stack: Box::new(stack.clone()),
        })
    }
    fn to_agal_string(self, _: &Stack, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<interno>".to_string()))
    }
    fn to_agal_boolean(self, stack: &Stack, env: &Enviroment) -> Result<AgalBoolean, AgalThrow> {
        let value = env.get(TRUE_KEYWORD, stack.get_value());
        let value: &AgalValue = &value.as_ref().borrow();
        match value {
            AgalValue::Boolean(b) => Ok(b.clone()),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en booleano".to_string(),
                stack: Box::new(stack.clone()),
            }),
        }
    }
    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Iterable".to_string()),
            message: "El valor no es iterable".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    // utils
    fn to_agal_value(self, stack: &Stack, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }
    fn to_agal_console(self, stack: &Stack, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        let value = self.to_agal_string(stack, env);
        if value.is_err() {
            return value;
        }
        let value = value.ok().unwrap();
        let str = value.get_string();
        let data = format!("\x1b[36m{}\x1b[39m", str);
        Ok(AgalString::from_string(data))
    }

    fn binary_operation(
        &self,
        stack: &Stack,
        env: &Enviroment,
        operator: String,
        other: RefAgalValue,
    ) -> RefAgalValue {
        binary_operation_error(stack, operator, other, None)
    }

    // object methods
    fn get_object_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
        get_property_error(stack, env, key)
    }
    fn set_object_property(
        mut self,
        stack: &Stack,
        env: &Enviroment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        set_property_error(stack, env, key, "No se puede asignar".to_string())
    }
    fn delete_object_property(mut self, stack: &Stack, env: &Enviroment, key: String) {
        delete_property_error(stack, env, key);
    }
    // instance methods
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue;

    // values
    fn call(
        self,
        _: &Stack,
        _: &Enviroment,
        _: RefAgalValue,
        _: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        AgalValue::Never.to_ref()
    }
    fn construct(self, _: &Stack, _: &Enviroment, _: Vec<RefAgalValue>) -> RefAgalValue {
        AgalValue::Never.to_ref()
    }
}

pub fn set_property_error(
    stack: &Stack,
    env: &Enviroment,
    key: String,
    message: String,
) -> RefAgalValue {
    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad".to_string()),
        message: format!("No se puede obtener la propiedad {}: {}", key, message),
        stack: Box::new(stack.clone()),
    })
    .to_ref()
}
pub fn get_property_error(stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
    Rc::new(RefCell::new(AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad".to_string()),
        message: format!("No se puede obtener la propiedad {}", key),
        stack: Box::new(stack.clone()),
    })))
}
pub fn delete_property_error(stack: &Stack, env: &Enviroment, key: String) -> AgalValue {
    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Propiedad".to_string()),
        message: format!("No se puede eliminar la propiedad {}", key),
        stack: Box::new(stack.clone()),
    })
}

pub fn get_instance_property_error(
    stack: &Stack,
    env: &Enviroment,
    key: String,
    value: AgalValue,
) -> RefAgalValue {
    let rc_stack: Rc<RefCell<Stack>> = Rc::new(RefCell::new(stack.clone()));
    let rc_env: Rc<RefCell<Enviroment>> = Rc::new(RefCell::new(env.clone()));
    match key.as_str() {
        "aCadena" => {
            let key_clone = key.clone();
            let function = {
                let e_stack = Rc::clone(&rc_stack);
                let e_env = Rc::clone(&rc_env);
                move |_: Vec<RefAgalValue>| -> RefAgalValue {
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
            value.to_ref()
        }
        _ => get_property_error(stack, env, key),
    }
}

fn get_instance_property_value(
    stack: Rc<RefCell<Stack>>,
    env: Rc<RefCell<Enviroment>>,
    key: &str,
    value: &AgalValue,
) -> RefAgalValue {
    let stack = &stack.as_ref().borrow();
    let env: &Enviroment = &env.as_ref().borrow();
    match key {
        "aCadena" => {
            let data = value
                .clone()
                .to_agal_string(stack, &env.clone().crate_child());
            let data = match data {
                Ok(s) => AgalValue::String(s),
                Err(e) => AgalValue::Throw(e),
            };
            Rc::new(RefCell::new(data))
        }
        _ => get_property_error(stack, env, key.to_string()),
    }
}
pub fn binary_operation_error(
    stack: &Stack,
    operator: String,
    left: RefAgalValue,
    rigth: Option<RefAgalValue>,
) -> RefAgalValue {
    let left = left.borrow();
    let message = match rigth {
        Some(r) => format!(
            "No se puede realizar la operacion {} {} {}",
            left.get_type(),
            operator,
            r.borrow().get_type()
        ),
        None => format!(
            "No se puede realizar la operacion {} con {}",
            operator,
            left.get_type()
        ),
    };

    AgalValue::Throw(AgalThrow::Params {
        type_error: ErrorNames::CustomError("Error Operacion".to_string()),
        message,
        stack: Box::new(stack.clone()),
    })
    .to_ref()
}
