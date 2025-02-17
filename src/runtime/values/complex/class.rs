use std::{cell::RefCell, collections::HashMap, rc::Rc};

use parser::util::{OpRefValue, RefValue};

use crate::{
  colors,
  runtime::{
    self,
    values::{
      self, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  Modules,
};

use super::AgalComplex;

type RefHasMap<Value> = Rc<RefCell<HashMap<String, Value>>>;
fn ref_hash_map<T: Clone>() -> RefHasMap<T> {
  Rc::new(RefCell::new(HashMap::new()))
}
#[derive(Clone, Debug)]
pub struct AgalClassProperty {
  pub is_public: bool,
  pub is_static: bool,
  pub value: values::DefaultRefAgalValue,
}
#[derive(Clone, Debug)]
pub struct AgalPrototype {
  instance_properties: RefHasMap<AgalClassProperty>,
  super_instance: Option<values::RefAgalValue<AgalPrototype>>,
}

impl AgalPrototype {
  pub fn new(
    instance_properties: RefHasMap<AgalClassProperty>,
    super_instance: Option<values::RefAgalValue<AgalPrototype>>,
  ) -> Self {
    Self {
      instance_properties,
      super_instance,
    }
  }
  pub fn get(&self, key: &str) -> Option<AgalClassProperty> {
    if self.instance_properties.borrow().contains_key(key) {
      self.instance_properties.borrow().get(key).cloned()
    } else if let Some(p) = &self.super_instance {
      p.borrow().get(key)
    } else {
      None
    }
  }
}
impl traits::ToAgalValue for AgalPrototype {
  fn to_value(self) -> AgalValue {
    AgalComplex::SuperInstance(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalPrototype {
  fn get_name(&self) -> String {
    "Clase".to_string()
  }
  fn to_agal_string(&self,stack: runtime::RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<instancia super>".to_string(),
    ))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::CYAN))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
  
  fn equals(&self, other: &Self) -> bool {
        todo!()
    }
  
  fn less_than(&self, other: &Self) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct AgalClass {
  name: String,
  extend_of: Option<values::RefAgalValue<AgalClass>>,
  static_properties: RefHasMap<AgalClassProperty>,
  instance: values::RefAgalValue<AgalPrototype>,
}

impl AgalClass {
  pub fn new(
    name: String,
    properties: Vec<(String, AgalClassProperty)>,
    extend_of: Option<values::RefAgalValue<AgalClass>>,
  ) -> Self {
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
      let value = class.un_ref();
      Some(value.instance.clone())
    } else {
      None
    };

    let instance = AgalPrototype::new(instance_properties, super_instance).as_ref();

    Self {
      name,
      static_properties,
      instance,
      extend_of,
    }
  }
  pub fn constructor(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    this: values::RefAgalValue<super::AgalObject>,
    args: Vec<values::DefaultRefAgalValue>,
    modules_manager: RefValue<Modules>,
  ) -> values::DefaultRefAgalValue {
    if let Some(class) = &self.extend_of {
      let value = class.un_ref();
      value.constructor(
        stack.clone(),
        env.clone(),
        this.clone(),
        args.clone(),
        modules_manager.clone(),
      );
    }
    let instance = self.instance.borrow();
    let instance_properties = instance.instance_properties.borrow();
    let constructor = instance_properties.get("constructor");
    let this_value = this.borrow().clone().to_ref_value();
    if let Some(property) = constructor {
      let property_value = property.value.un_ref();
      property_value
        .clone()
        .call(stack, env, this_value.clone(), args, modules_manager);
    }
    this_value
  }
}

impl traits::ToAgalValue for AgalClass {
  fn to_value(self) -> AgalValue {
    AgalComplex::Class(self.as_ref()).to_value()
  }
}
impl traits::AgalValuable for AgalClass {
  fn get_name(&self) -> String {
    "Clase".to_string()
  }
  fn to_agal_string(&self,stack: runtime::RefStack) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<instancia super>".to_string(),
    ))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::CYAN))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &mut self,
    stack: runtime::RefStack,
    env: runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
  
  fn equals(&self, other: &Self) -> bool {
        todo!()
    }
  
  fn less_than(&self, other: &Self) -> bool {
        todo!()
    }
}
