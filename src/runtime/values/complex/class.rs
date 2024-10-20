use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::AgalObject;
use crate::runtime::{env::RefEnvironment, AgalValuable, AgalValue, RefAgalValue, Stack};
use parser::util::{OpRefValue, RefValue};
pub type RefHasMap<Value> = Rc<RefCell<HashMap<String, Value>>>;

fn ref_hash_map<T: Clone>() -> RefHasMap<T> {
    Rc::new(RefCell::new(HashMap::new()))
}

#[derive(Clone, PartialEq)]
pub struct AgalClassProperty {
    pub is_public: bool,
    pub is_static: bool,
    pub value: RefAgalValue,
}

#[derive(Clone, PartialEq)]
pub struct AgalClass {
    name: String,
    extend_of: OpRefValue<AgalClass>,
    static_properties: RefHasMap<AgalClassProperty>,
    instance_properties: RefHasMap<AgalClassProperty>,
}

impl AgalClass {
    pub fn new(
        name: String,
        properties: Vec<(String, AgalClassProperty)>,
        extend_of: OpRefValue<AgalClass>,
    ) -> AgalClass {
        let static_properties = ref_hash_map();
        let instance_properties = ref_hash_map();
        for property in properties.iter() {
            let mut properties = if property.1.is_static {
                static_properties.as_ref().borrow_mut()
            } else {
                instance_properties.as_ref().borrow_mut()
            };

            properties.insert(property.0.clone(), property.1.clone());
        }
        AgalClass {
            name,
            static_properties,
            instance_properties,
            extend_of,
        }
    }
    pub fn constructor(
        self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefValue<AgalObject>,
        args: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        if let Some(class) = self.extend_of {
            let value = class.as_ref().borrow();
            value
                .clone()
                .constructor(stack, env.clone(), this.clone(), args.clone());
        }
        let sp = self.static_properties.borrow();
        let constructor = sp.get("constructor");
        if let Some(p) = constructor {
            let tv = this.as_ref().borrow();
            let pv = p.value.as_ref().borrow();
            pv.clone().call(stack, env, tv.clone().to_ref_value(), args);
        }
        let object = this.borrow();
        object.clone().to_ref_value()
    }
}
impl AgalValuable for AgalClass {
    fn to_value(self) -> AgalValue {
        AgalValue::Class(self)
    }
    fn get_instance_property(
        self,
        _: &crate::runtime::Stack,
        env: crate::runtime::env::RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        if key == "constructor" {
            return self.to_ref_value();
        }
        let this = self.clone();
        let prop = this.static_properties.borrow();
        let prop = prop.get(&key);
        if let Some(property) = prop {
            if property.is_public && property.is_static {
                property.clone().value
            } else if !property.is_public
                && property.is_static
                && env.borrow().clone().use_private()
            {
                property.clone().value
            } else {
                AgalValue::Never.as_ref()
            }
        } else {
            AgalValue::Never.as_ref()
        }
    }

    fn call(
        self,
        stack: &Stack,
        env: RefEnvironment,
        _: RefAgalValue,
        args: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        let this = Rc::new(RefCell::new(AgalObject::from_prototype(
            self.clone().instance_properties,
        )));
        self.constructor(stack, env, this, args)
    }
}
