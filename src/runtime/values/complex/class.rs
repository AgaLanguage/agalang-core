use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::AgalObject;
use crate::{runtime::{
    env::RefEnvironment, AgalString, AgalThrow, AgalValuable, AgalValue, RefAgalValue, Stack,
}, Modules};
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
pub struct AgalPrototype {
    instance_properties: RefHasMap<AgalClassProperty>,
    super_instance: OpRefValue<AgalPrototype>,
}

impl AgalPrototype {
    pub fn new(
        instance_properties: RefHasMap<AgalClassProperty>,
        super_instance: OpRefValue<AgalPrototype>,
    ) -> AgalPrototype {
        AgalPrototype {
            instance_properties,
            super_instance,
        }
    }
    pub fn as_ref(self) -> RefValue<AgalPrototype> {
        Rc::new(RefCell::new(self))
    }
}

impl AgalValuable for AgalPrototype {
    fn to_value(self) -> AgalValue {
        AgalValue::SuperInstance(self)
    }
    fn to_agal_console(self, stack: &Stack, env: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string("\x1b[36m<instancia super>\x1b[39m".to_string()))
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        if key == "super".to_string() {
            return if let Some(s) = self.super_instance {
                s.borrow().clone().to_ref_value()
            } else {
                AgalValue::Never.as_ref()
            };
        }
        let prop = self.instance_properties.borrow();
        let prop = prop.get(&key);
        if let Some(property) = prop {
            if property.is_public {
                property.clone().value
            } else if !property.is_public && env.borrow().clone().use_private() {
                property.clone().value
            } else {
                AgalValue::Never.as_ref()
            }
        } else if let Some(s) = self.super_instance {
            let prop = s.borrow().clone();
            prop.get_instance_property(stack, env, key)
        } else {
            AgalValue::Never.as_ref()
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct AgalClass {
    name: String,
    extend_of: OpRefValue<AgalClass>,
    static_properties: RefHasMap<AgalClassProperty>,
    instance: RefValue<AgalPrototype>,
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
            if property.0 == "super" {
                continue;
            }
            let mut properties = if property.1.is_static {
                static_properties.as_ref().borrow_mut()
            } else {
                instance_properties.as_ref().borrow_mut()
            };

            properties.insert(property.0.clone(), property.1.clone());
        }
        let super_instance = if let Some(class) = &extend_of {
            let value = class.as_ref().borrow();
            Some(value.instance.clone())
        } else {
            None
        };

        let instance = AgalPrototype::new(instance_properties, super_instance).as_ref();

        AgalClass {
            name,
            static_properties,
            instance,
            extend_of,
        }
    }
    pub fn constructor(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        this: RefValue<AgalObject>,
        args: Vec<RefAgalValue>,
        modules_manager:&Modules
    ) -> RefAgalValue {
        if let Some(class) = &self.extend_of {
            let value = class.as_ref().borrow();
            value.constructor(stack, env.clone(), this.clone(), args.clone(), modules_manager);
        }
        let instance = self.instance.borrow();
        let instance_properties = instance.instance_properties.borrow();
        let constructor = instance_properties.get("constructor");
        if let Some(property) = constructor {
            let this_value = this.as_ref().borrow();
            let property_value = property.value.as_ref().borrow();
            property_value
                .clone()
                .call(stack, env, this_value.clone().to_ref_value(), args, modules_manager);
        }
        let object = this.borrow();
        object.clone().to_ref_value()
    }
}
impl AgalValuable for AgalClass {
    fn to_value(self) -> AgalValue {
        AgalValue::Class(self)
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[36m<clase '{}'>\x1b[39m",
            self.name
        )))
    }
    fn get_instance_property(
        self,
        _: &crate::runtime::Stack,
        env: crate::runtime::env::RefEnvironment,
        key: String,
    ) -> RefAgalValue {
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
        modules_manager:&Modules
    ) -> RefAgalValue {
        let o = AgalObject::from_prototype(self.clone().instance);
        let this = Rc::new(RefCell::new(o.clone()));
        self.constructor(stack, env, this, args, modules_manager);
        o.to_ref_value()
    }
}
