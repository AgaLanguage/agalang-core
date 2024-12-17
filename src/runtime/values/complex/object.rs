use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::runtime::{
    env::RefEnvironment, get_instance_property_error, AgalComplex, AgalPrototype, AgalString,
    AgalThrow, AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, RefHasMap, Stack,
};

pub type AgalHashMap<Value> = std::collections::HashMap<String, Value>;
pub type RefAgalHashMap<'a> = Rc<RefCell<AgalHashMap<RefAgalValue<'a>>>>;
pub type RefAgalProto<'a> = Rc<RefCell<AgalPrototype<'a>>>;

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub struct AgalObject<'a>(RefAgalHashMap<'a>, Option<RefAgalProto<'a>>);
impl<'a> AgalObject<'a> {
    pub fn from_hashmap(hashmap: RefAgalHashMap<'a>) -> Self {
        Self(hashmap, None)
    }
    pub fn from_hashmap_with_prototype(
        hashmap: RefAgalHashMap<'a>,
        prototype: RefAgalProto<'a>,
    ) -> Self {
        Self(hashmap, Some(prototype))
    }
    pub fn from_prototype(hashmap: RefAgalProto) -> AgalObject {
        AgalObject(Rc::new(RefCell::new(HashMap::new())), Some(hashmap))
    }
    pub fn get_hashmap(&'a self) -> Ref<AgalHashMap<RefAgalValue>> {
        self.0.as_ref().borrow()
    }
    pub fn get_prototype(&'a self) -> Option<Ref<AgalPrototype>> {
        if let Some(a) = &self.1 {
            Some(a.as_ref().borrow())
        } else {
            None
        }
    }
}
impl<'a> AgalValuable<'a> for AgalObject<'a> {
    fn get_keys(&'a self) -> Vec<String> {
        let hashmap = self.get_hashmap();
        hashmap.keys().cloned().collect()
    }
    fn to_agal_string(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("<Objeto>")))
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        let mut result = String::new();
        for (i, (key, value)) in self.0.as_ref().borrow().iter().enumerate() {
            let data = value
                .as_ref()
                .borrow()
                .clone()
                .to_agal_value(stack, env.clone());
            let str = if let Ok(st) = &data {
                st.get_string()
            } else {
                return data;
            };
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!("{}: {}", key, str));
        }
        Ok(AgalString::from_string(format!("{{ {result} }}")))
    }
    fn to_value(self) -> AgalValue<'a> {
        AgalComplex::Object(self).to_value()
    }
    fn get_object_property(&'a self, _: &Stack, _: RefEnvironment, key: String) -> RefAgalValue {
        let value = self.get_hashmap();
        let value = value.get(&key);
        if value.is_none() {
            return AgalValue::Never.as_ref();
        }
        value.unwrap().clone()
    }
    fn set_object_property(
        &self,
        _: &Stack,
        _: RefEnvironment,
        key: String,
        value: RefAgalValue<'a>,
    ) -> RefAgalValue<'a> {
        self.0.borrow_mut().insert(key, value.clone());
        value
    }
    fn get_instance_property(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let this = self.clone();
        let proto = this.get_prototype();
        if proto.is_none() {
            let value = AgalComplex::Object(*self).to_value();
            return get_instance_property_error(stack, env.clone(), key, &value);
        }
        let value = proto.unwrap();
        value.clone().get_instance_property(stack, env, key)
    }
}
