use std::{cell::RefCell, rc::Rc};

use crate::{
    runtime::{env::RefEnvironment, AgalThrow, AgalValuable, AgalValue, RefAgalValue, Stack},
    util::OpRefValue,
};

use super::AgalFunction;

pub type AgalHashMap = std::collections::HashMap<String, AgalClassProperty>;
pub type RefAgalHashMap = Rc<RefCell<AgalHashMap>>;

#[derive(Clone, PartialEq)]
pub enum AgalClassPropertyManager {
    Getter(AgalFunction),
    Setter(AgalFunction),
    GetterSetter(/*getter*/ AgalFunction, /*setter*/ AgalFunction),
    Value,
}
#[derive(Clone,PartialEq)]
pub struct AgalClassProperty {
    pub is_const: bool,
    pub is_static: bool,
    pub is_public: bool,
    pub manager: AgalClassPropertyManager,
    pub value: RefAgalValue,
}

impl AgalClassProperty {
    fn get(self, stack: &Stack, env: RefEnvironment, value: RefAgalValue) -> RefAgalValue {
        match self.manager {
            AgalClassPropertyManager::Getter(f) => f.call(stack, env, value, vec![]),
            AgalClassPropertyManager::GetterSetter(f, _) => f.call(stack, env, value, vec![]),
            AgalClassPropertyManager::Setter(_) => AgalValue::Never.as_ref(),
            AgalClassPropertyManager::Value => self.value,
        }
    }
    fn set(
        mut self,
        new_value: RefAgalValue,
        stack: &Stack,
        env: RefEnvironment,
        value: RefAgalValue,
    ) -> RefAgalValue {
        let result = match self.manager {
            AgalClassPropertyManager::Getter(_) => AgalValue::Never.as_ref(),
            AgalClassPropertyManager::GetterSetter(_, f) => f.call(stack, env, value, vec![]),
            AgalClassPropertyManager::Setter(f) => f.call(stack, env, value, vec![]),
            AgalClassPropertyManager::Value => new_value,
        };
        if self.is_const {
            return AgalThrow::Params {
                type_error: crate::internal::ErrorNames::CustomError("Error de Lectura"),
                message: "Las constantes no pueden ser reaccignadas".to_string(),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        self.value = result.clone();
        result
    }
}

#[derive(Clone, PartialEq)]
pub struct AgalClass {
    name: String,
    extend_of: OpRefValue<AgalValue>,
    properties: RefAgalHashMap,
}

impl AgalClass {
    pub fn new(name: String, properties: RefAgalHashMap, extend_of: OpRefValue<AgalValue>) -> AgalClass{
        AgalClass {
            name,
            properties,
            extend_of,
        }
    }
}
impl AgalValuable for AgalClass {
    fn to_value(self) -> AgalValue {
        AgalValue::Class(self)
    }
    fn get_instance_property(
        self,
        stack: &crate::runtime::Stack,
        env: crate::runtime::env::RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let this = self.clone();
        let prop = this.properties.borrow();
        let prop = prop.get(&key);
        if let Some(property) = prop {
            if property.is_public && property.is_static {
                property.clone().get(stack, env.clone(), self.to_ref_value())
            } else if !property.is_public && property.is_static && env.borrow().clone().use_private() {
                property.clone().get(stack, env, self.to_ref_value())
            } else {
                AgalValue::Never.as_ref()
            }
        } else {
            AgalValue::Never.as_ref()
        }
    }
}
