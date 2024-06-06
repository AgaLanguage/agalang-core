use super::{
    env::{FALSE_KEYWORD, NOTHING_KEYWORD, NULL_KEYWORD, TRUE_KEYWORD},
    Enviroment, Stack,
};
use crate::{frontend::ast::Node, internal::ErrorNames};
use std::collections::HashMap;

pub mod primitive;
pub use primitive::*;
pub mod complex;
pub use complex::*;
pub mod internal;
pub use internal::*;

#[derive(Clone)]
pub enum AgalValue {
    Number(AgalNumber),
    String(AgalString),
    Boolean(AgalBoolean),
    Null,
    Never,
    Throw(AgalThrow),
    Function(Vec<String>, Vec<Node>, Enviroment),
    Array(Vec<AgalValue>),
    Object(HashMap<String, AgalValue>),
    NativeFunction(fn(Vec<AgalValue>) -> AgalValue),
    Return(Box<AgalValue>),
}
impl AgalValuable for AgalValue {
    fn to_agal_number(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalNumber, AgalThrow> {
        match self {
            AgalValue::Number(n) => Ok(n),
            AgalValue::String(s) => s.to_agal_number(stack, env),
            AgalValue::Boolean(b) => b.to_agal_number(stack, env),
            AgalValue::Null => Ok(AgalNumber::new(0f64)),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en numero".to_string(),
                stack,
            }),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en numero".to_string(),
                stack,
            }),
        }
    }
    fn to_agal_string(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        match self {
            AgalValue::Number(n) => n.to_agal_string(stack, env),
            AgalValue::String(s) => Ok(s),
            AgalValue::Boolean(b) => b.to_agal_string(stack, env),
            AgalValue::Null => Ok(AgalString::from_string(NULL_KEYWORD.to_string())),
            AgalValue::Never => Ok(AgalString::from_string(NOTHING_KEYWORD.to_string())),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en cadena".to_string(),
                stack,
            }),
        }
    }
    fn to_agal_boolean(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalBoolean, AgalThrow> {
        match self {
            AgalValue::Number(n) => n.to_agal_boolean(stack, env),
            AgalValue::String(s) => s.to_agal_boolean(stack, env),
            AgalValue::Boolean(b) => Ok(b),
            AgalValue::Null => env.get(FALSE_KEYWORD, &Node::None).to_agal_boolean(stack, env),
            AgalValue::Never => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en booleano".to_string(),
                stack,
            }),
            AgalValue::Throw(t) => Err(t),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en booleano".to_string(),
                stack,
            }),
        }
    }
}

impl std::fmt::Display for AgalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgalValue::Number(n) => write!(f, "{}", n.to_f64()),
            AgalValue::String(s) => write!(
                f,
                "{}",
                s.get_vec().iter().map(|c| c.to_char()).collect::<String>()
            ),
            AgalValue::Boolean(b) => write!(
                f,
                "{}",
                if b.to_bool() {
                    TRUE_KEYWORD
                } else {
                    FALSE_KEYWORD
                }
            ),
            AgalValue::Null => write!(f, "nulo"),
            AgalValue::Never => write!(f, "nada"),
            AgalValue::Throw(t) => write!(f, "{}", t),
            AgalValue::Function(_, _, _) => write!(f, "Funcion"),
            AgalValue::Array(_) => write!(f, "[..]"),
            AgalValue::Object(_) => write!(f, "{{..}}"),
            AgalValue::NativeFunction(_) => write!(f, "Funcion nativa"),
            AgalValue::Return(r) => write!(f, "Retorno: {}", r),
        }
    }
}

pub trait AgalValuable<T: AgalValuable = Self>
where
    Self: Sized + Clone,
{
    // types
    fn to_agal_number(self, stack: Box<Stack>, _: &Enviroment) -> Result<AgalNumber, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Parseo".to_string()),
            message: "No se pudo convertir en numero".to_string(),
            stack,
        })
    }
    fn to_agal_string(self, _: Box<Stack>, _: &Enviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("<interno>".to_string()))
    }
    fn to_agal_boolean(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalBoolean, AgalThrow> {
        let value = env.get(TRUE_KEYWORD, stack.get_value());
        match value {
            AgalValue::Boolean(b) => Ok(b),
            _ => Err(AgalThrow::Params {
                type_error: ErrorNames::CustomError("Error Parseo".to_string()),
                message: "No se pudo convertir en booleano".to_string(),
                stack,
            }),
        }
    }
    fn to_agal_array(self, stack: Box<Stack>) -> Result<AgalArray<T>, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: ErrorNames::CustomError("Error Iterable".to_string()),
            message: "El valor no es iterable".to_string(),
            stack,
        })
    }

    // utils
    fn to_agal_value(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }
    fn to_agal_console(self, stack: Box<Stack>, env: &Enviroment) -> Result<AgalString, AgalThrow> {
        let value = self.to_agal_string(stack, env);
        if value.is_err() {
            return value;
        }
        let value = value.ok().unwrap();
        let str = value
            .get_vec()
            .iter()
            .map(|c| c.to_char())
            .collect::<String>();
        let data = format!("\x1b[36m{}\x1b[39m", str);
        Ok(AgalString::from_string(data))
    }

    // value
    fn get(&self, _: String) -> AgalValue {
        AgalValue::Never
    }
    fn set(&self, _: String, _: AgalValue) -> AgalValue {
        AgalValue::Never
    }
    fn delete(&self, _: String) -> AgalValue {
        AgalValue::Never
    }
    fn call(&self, _: AgalValue, _: Vec<AgalValue>) -> AgalValue {
        AgalValue::Never
    }
    fn construct(&self, _: Vec<AgalValue>) -> AgalValue {
        AgalValue::Never
    }
}
