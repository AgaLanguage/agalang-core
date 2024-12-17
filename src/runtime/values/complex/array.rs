use std::{cell::RefCell, rc::Rc};

use parser::util::RefValue;

use crate::{
    runtime::{
        env::RefEnvironment, get_instance_property_error, get_property_error, AgalByte,
        AgalComplex, AgalNumber, AgalString, AgalThrow, AgalValuable, AgalValuableManager,
        AgalValue, RefAgalValue, Stack,
    },
    Modules,
};

pub type AgalVec<'a> = Vec<RefAgalValue<'a>>;
#[derive(Clone, PartialEq)]
pub struct AgalArray<'a>(RefValue<AgalVec<'a>>);
impl<'a> AgalArray<'a> {
    fn new(vec: AgalVec<'a>) -> Self {
        Self(Rc::new(RefCell::new(vec)))
    }
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let mut vec = Vec::new();
        for byte in buffer {
            vec.push(AgalByte::new(*byte).to_ref_value());
        }
        Self::new(vec)
    }
    pub fn from_vec(vec: AgalVec<'a>) -> Self {
        Self::new(vec)
    }
    pub fn get_vec(&self) -> RefValue<AgalVec<'a>> {
        self.0.clone()
    }
    pub fn get_buffer(&self, stack: &Stack) -> Result<Vec<u8>, AgalThrow> {
        let mut buffer = vec![];
        let vec: &AgalVec = &self.0.borrow();
        for value in vec {
            let byte = value.as_ref().borrow().to_agal_byte(stack);
            if let Err(value) = byte {
                return Err(value);
            }
            buffer.push(byte?.to_u8());
        }
        Ok(buffer)
    }
}
impl<'a> AgalValuable<'a> for AgalArray<'a> {
    fn get_length(&self) -> usize {
        self.get_vec().borrow().len()
    }
    fn to_agal_console(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for (i, value) in self.get_vec().borrow().iter().enumerate() {
            let str = value
                .as_ref()
                .borrow()
                .to_agal_value(stack, env.clone());
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str.get_string();
            let str = if i == 0 { str } else { &format!(", {str}") };
            result.push_str(str);
        }
        Ok(AgalString::from_string(format!("[ {result} ]").as_str()))
    }
    fn to_agal_array(&self, _: &Stack) -> Result<&AgalArray<'a>, AgalThrow> {
        Ok(self)
    }
    fn to_value(self) -> AgalValue<'a> {
        AgalComplex::Array(self).to_value()
    }
    fn to_agal_string(&self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for value in self.get_vec().borrow().iter() {
            let str = value.as_ref().borrow().to_agal_string(
                stack,
                env.as_ref().borrow().clone().crate_child(false).as_ref(),
            );
            if str.is_err() {
                return str;
            }
            let str = str.ok().unwrap();
            let str = str.get_string();
            result.push_str(&str);
        }
        Ok(AgalString::from_string(&result))
    }
    fn get_instance_property(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        match key.as_str() {
            "unir" => crate::runtime::AgalNativeFunction {
                name: "unir".to_string(),
                func: Rc::new(move |args, stack, env, _, _| {
                    let sep = args.get(0);
                    let sep = if let Some(s) = sep {
                        s.borrow().clone()
                    } else {
                        let ref value = AgalValue::Never;
                        return value.to_ref_value();
                    };
                    let sep = sep.to_agal_string(stack, env.clone());
                    let sep = if let Ok(s) = &sep {
                        s.get_string()
                    } else if let Err(e) = sep {
                        return e.to_ref_value();
                    } else {
                        ""
                    };
                    let mut result = String::new();
                    for (i, value) in (&self).0.borrow().iter().enumerate() {
                        let data = value.borrow().to_agal_string(stack, env.clone());
                        let str = if let Ok(s) = &data {
                            s.get_string()
                        } else if let Err(e) = data {
                            return e.to_ref_value();
                        } else {
                            ""
                        };
                        if i > 0 {
                            result.push_str(sep);
                        }
                        result.push_str(str);
                    }
                    AgalString::from_string(&result).to_ref_value()
                }),
            }
            .to_ref_value(),
            "agregar" => crate::runtime::AgalNativeFunction {
                name: "agregar".to_string(),
                func: Rc::new(move |args, stack, env, _, _| {
                    let mut vec = (&self).0.borrow_mut();
                    for arg in args.iter() {
                        vec.push(arg.clone());
                    }
                    self.clone().to_ref_value()
                }),
            }
            .to_ref_value(),
            "largo" => AgalNumber::new(self.get_length() as f64).to_ref_value(),
            _ => get_instance_property_error(stack, env, key, &self.to_value()),
        }
    }
    fn get_object_property(&'a self, stack: &Stack, env: RefEnvironment<'a>, key: String) -> RefAgalValue<'a> {
        let int = key.parse::<usize>();
        if int.is_err() {
            return get_property_error(stack, env, key);
        }
        let int = int.unwrap();
        let value = self.0.borrow();
        let value = value.get(int);
        value.unwrap_or(&AgalValue::Never.as_ref()).clone()
    }
}
