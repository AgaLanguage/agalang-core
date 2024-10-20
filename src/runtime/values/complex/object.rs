use std::{
  cell::{Ref, RefCell}, collections::HashMap, rc::Rc
};

use crate::runtime::{env::RefEnvironment, get_instance_property_error, AgalString, AgalThrow, AgalValuable, AgalValue, RefAgalValue, Stack};

use super::{AgalClassProperty, RefHasMap};

pub type AgalHashMap<Value> = std::collections::HashMap<String, Value>;
pub type RefAgalHashMap = Rc<RefCell<AgalHashMap<RefAgalValue>>>;

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub struct AgalObject(RefAgalHashMap, Option<RefHasMap<AgalClassProperty>>);
impl AgalObject {
    pub fn from_hashmap(hashmap: RefAgalHashMap) -> AgalObject {
        AgalObject(hashmap, None)
    }
    pub fn from_hashmap_with_prototype(hashmap: RefAgalHashMap, prototype: RefHasMap<AgalClassProperty>) -> AgalObject {
        AgalObject(
            hashmap,
            Some(prototype),
        )
    }
    pub fn from_prototype(hashmap: RefHasMap<AgalClassProperty>) -> AgalObject {
        AgalObject(Rc::new(RefCell::new(HashMap::new())), Some(hashmap))
    }
    pub fn get_hashmap(&self) -> Ref<AgalHashMap<RefAgalValue>> {
        self.0.as_ref().borrow()
    }
    pub fn get_prototype(&self) -> Option<Ref<AgalHashMap<AgalClassProperty>>> {
        if self.1.is_none() {
            return None;
        }
        Some(self.1.as_ref().unwrap().borrow())
    }
}
impl AgalValuable for AgalObject {
    fn get_keys(self) -> Vec<String> {
        let hashmap = self.get_hashmap();
        hashmap.keys().cloned().collect()
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let string = "\x1b[36m[Objeto]\x1b[39m".to_string();
        Ok(AgalString::from_string(string))
    }
    fn to_value(self) -> AgalValue {
        AgalValue::Object(self)
    }
    fn get_object_property(self, _: &Stack, _: RefEnvironment, key: String) -> RefAgalValue {
        let value = self.get_hashmap();
        let value = value.get(&key);
        if value.is_none() {
            return AgalValue::Never.as_ref();
        }
        value.unwrap().clone()
    }
    fn set_object_property(
        self,
        _: &Stack,
        _: RefEnvironment,
        key: String,
        value: RefAgalValue,
    ) -> RefAgalValue {
        self.0.borrow_mut().insert(key, value.clone());
        value
    }
    fn get_instance_property(self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
        let this = self.clone();
        let proto = this.get_prototype();
        if proto.is_none() {
            let value = AgalValue::Object(self);
            return get_instance_property_error(stack, env, key, value);
        }
        let value = proto.unwrap();
        let value = value.get(&key);
        if value.is_none() {
            return AgalValue::Never.as_ref();
        }
        value.unwrap().clone().value
    }
}
