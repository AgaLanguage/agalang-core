use std::{cell::RefCell, collections::HashMap, fmt::format, rc::Rc};

use crate::{
  functions_names, libraries, parser,
  runtime::{
    self,
    values::{
      self, error_message, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  util::{self, OnError as _},
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
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<instancia super>".to_string(),
    ))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::CYAN),
    )
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    match &self.get(functions_names::TO_AGAL_BYTE) {
      Some(prop) => prop
        .value
        .call(
          stack.clone(),
          self.clone().to_ref_value(),
          vec![],
          modules.clone(),
        )?
        .to_agal_byte(stack, modules),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_BYTE.into(),
        stack,
      }),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    match &self.get(functions_names::TO_AGAL_BOOLEAN) {
      Some(prop) => prop
        .value
        .call(
          stack.clone(),
          self.clone().to_ref_value(),
          vec![],
          modules.clone(),
        )?
        .to_agal_boolean(stack, modules),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_BOOLEAN.into(),
        stack,
      }),
    }
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    match &self.get(functions_names::TO_AGAL_ARRAY) {
      Some(prop) => prop
        .value
        .call(
          stack.clone(),
          self.clone().to_ref_value(),
          vec![],
          modules.clone(),
        )?
        .to_agal_array(stack, modules),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_ARRAY.into(),
        stack,
      }),
    }
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match &self.get(functions_names::BINARY_OPERATION) {
      Some(prop) => prop.value.call(
        stack.clone(),
        self.clone().to_ref_value(),
        vec![],
        modules.clone(),
      ),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match self.get(key) {
      Some(prop) => {
        if prop.is_public {
          Ok(prop.value)
        } else if stack.env().use_private() {
          Ok(prop.value)
        } else {
          Err(format!(
            "La propiedad '{key}' es privada y no se puede acceder"
          ))
        }
      }
      None => Err(format!("No existe la propiedad '{key}'")),
    }
    .on_error(|message| internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message,
      stack,
    })
  }

  fn call(
    &self,
    stack: runtime::RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    match &self.get(functions_names::CALL) {
      Some(prop) => prop.value.call(
        stack.clone(),
        self.clone().to_ref_value(),
        vec![],
        modules.clone(),
      ),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::CALL.into(),
        stack,
      }),
    }
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    match &self.get(functions_names::TO_AGAL_NUMBER) {
      Some(prop) => prop
        .value
        .call(
          stack.clone(),
          self.clone().to_ref_value(),
          vec![],
          modules.clone(),
        )?
        .to_agal_number(stack, modules),
      None => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_NUMBER.into(),
        stack,
      }),
    }
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
    modules_manager: libraries::RefModules,
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
    let constructor = instance_properties.get(functions_names::CONSTRUCTOR);
    let this_value = this.borrow().clone().to_ref_value();
    if let Some(property) = constructor {
      let property_value = property.value.un_ref();
      property_value
        .clone()
        .call(stack, this_value.clone(), args, modules_manager);
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
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<instancia super>".to_string(),
    ))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::CYAN),
    )
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(primitive::AgalBoolean::True)
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn call(
    &self,
    stack: runtime::RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    Rc::as_ptr(&self.static_properties) == Rc::as_ptr(&self.static_properties)
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}
