use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  fmt::Debug,
  rc::Rc,
};

use crate::parser;

use super::values::{DefaultRefAgalValue, ResultAgalValue};
use super::{
  stack::RefStack,
  values::{
    internal, primitive,
    traits::{self, AgalValuable as _, ToAgalValue as _},
    AgalValue,
  },
};

#[derive(Clone, Debug)]
pub struct Environment {
  in_class: bool,
  parent: Option<RefEnvironment>,
  variables: Rc<RefCell<HashMap<String, DefaultRefAgalValue>>>,
  constants: Rc<RefCell<HashSet<String>>>,
}
pub const TRUE_KEYWORD: &str = "cierto";
pub const FALSE_KEYWORD: &str = "falso";
pub const NULL_KEYWORD: &str = "nulo";
pub const NOTHING_KEYWORD: &str = "nada";
pub const THIS_KEYWORD: &str = "este";
pub const SUPER_KEYWORD: &str = "super";
const KEYWORDS: [&str; 6] = [
  TRUE_KEYWORD,
  FALSE_KEYWORD,
  NULL_KEYWORD,
  NOTHING_KEYWORD,
  THIS_KEYWORD,
  SUPER_KEYWORD,
];
impl Environment {
  pub fn get_default() -> Environment {
    let mut env = Environment {
      in_class: false,
      parent: None,
      variables: Rc::new(RefCell::new(HashMap::new())),
      constants: Rc::new(RefCell::new(HashSet::new())),
    };
    env.variables.borrow_mut().insert(
      TRUE_KEYWORD.to_string(),
      primitive::AgalBoolean::True.to_ref_value(),
    );
    env.variables.borrow_mut().insert(
      FALSE_KEYWORD.to_string(),
      primitive::AgalBoolean::False.to_ref_value(),
    );
    env
      .variables
      .borrow_mut()
      .insert(NULL_KEYWORD.to_string(), AgalValue::Null.as_ref());
    env
      .variables
      .borrow_mut()
      .insert(NOTHING_KEYWORD.to_string(), AgalValue::Never.as_ref());
    env
  }
  pub fn get_global(&self) -> RefEnvironment {
    let mut env = self.clone().as_ref();
    while env.has_parent() {
      env = env.get_parent();
    }
    env
  }
  pub fn as_ref(self) -> RefEnvironment {
    RefEnvironment(Rc::new(RefCell::new(self)))
  }
  pub fn get_this(&self, stack: RefStack, node: &parser::Node) -> ResultAgalValue {
    self.get(stack, THIS_KEYWORD, node)
  }
  pub fn use_private(self) -> bool {
    if self.in_class {
      true
    } else if let Some(p) = self.parent {
      p.use_private()
    } else {
      false
    }
  }
  pub fn crate_child(self, in_class: bool) -> Self {
    Self {
      in_class,
      parent: Some(self.as_ref()),
      variables: Rc::new(RefCell::new(HashMap::new())),
      constants: Rc::new(RefCell::new(HashSet::new())),
    }
  }
  pub fn is_keyword(&self, ref name: &str) -> bool {
    KEYWORDS.contains(name)
  }
  pub fn define(
    &mut self,
    stack: RefStack,
    name: &str,
    value: DefaultRefAgalValue,
    is_constant: bool,
    node: &parser::Node,
  ) -> ResultAgalValue {
    if self.is_keyword(name) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: "No se puede declarar una variable con el nombre de una palabra clave".to_string(),
        stack,
      });
    }
    if self._has(name) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: format!("La variable '{}' ya ha sido declarada", name),
        stack,
      });
    }
    if is_constant {
      self.constants.borrow_mut().insert(name.to_string());
    }
    self
      .variables
      .borrow_mut()
      .insert(name.to_string(), value.clone());
    Ok(value)
  }
  pub fn is_constant(&self, name: &str) -> bool {
    self.constants.borrow().contains(name)
  }
  pub fn delete(&self, name: &str) -> bool {
    self.variables.borrow_mut().remove(name).is_some()
  }
  pub fn assign(
    &mut self,
    stack: RefStack,
    name: &str,
    value: DefaultRefAgalValue,
    node: &parser::Node,
  ) -> ResultAgalValue {
    if self.is_keyword(name) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: "No se puede reasignar una palabra clave".to_string(),
        stack,
      });
    }
    if !self.has(name, node) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: format!("La variable {} no ha sido declarada", name),
        stack,
      });
    }
    if self.is_constant(name) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: "No se puede reasignar una constante".to_string(),
        stack,
      });
    }
    Ok(self.resolve(name, node).set(name, value))
  }
  pub fn set(&mut self, name: &str, value: DefaultRefAgalValue) -> DefaultRefAgalValue {
    let mut hashmap = self.variables.borrow_mut();
    if hashmap.contains_key(name) {
      hashmap.remove(name);
    }
    hashmap.insert(name.to_string(), value.clone());
    value
  }
  pub fn get(&self, stack: RefStack, name: &str, node: &parser::Node) -> ResultAgalValue {
    let env = self.resolve(name, node);
    if !env._has(name) {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::EnvironmentError,
        message: format!("La variable {} no ha sido declarada", name),
        stack,
      });
    }
    Ok(env.get_key(name).clone())
  }
  fn _has(&self, name: &str) -> bool {
    self.variables.borrow_mut().contains_key(name)
  }
  pub fn has(&self, name: &str, node: &parser::Node) -> bool {
    self.resolve(name, node)._has(name)
  }
  fn resolve(&self, name: &str, node: &parser::Node) -> RefEnvironment {
    if !self._has(name) && self.parent.is_some() {
      let a = self.parent.clone().unwrap();
      return a.resolve(name, node);
    }
    return self.clone().as_ref();
  }
}

#[derive(Clone, Debug)]
pub struct RefEnvironment(Rc<RefCell<Environment>>);

impl RefEnvironment {
  pub fn get_default() -> RefEnvironment {
    RefEnvironment(Rc::new(RefCell::new(Environment::get_default())))
  }
  pub fn un_ref(&self) -> Environment {
    self.0.borrow().clone()
  }
  pub fn get_global(&self) -> RefEnvironment {
    self.0.borrow().get_global().clone()
  }
  pub fn crate_child(&self, in_class: bool) -> Self {
    self.0.borrow().clone().crate_child(in_class).as_ref()
  }
  pub fn has_parent(&self) -> bool {
    self.0.borrow().parent.is_some()
  }
  pub fn get_parent(&self) -> RefEnvironment {
    self.0.borrow().parent.clone().unwrap()
  }
  pub fn use_private(&self) -> bool {
    self.0.borrow().clone().use_private()
  }
  pub fn set(&mut self, name: &str, value: DefaultRefAgalValue) -> DefaultRefAgalValue {
    self.0.borrow_mut().set(name, value)
  }
  pub fn define(
    &mut self,
    stack: RefStack,
    name: &str,
    value: DefaultRefAgalValue,
    is_constant: bool,
    node: &parser::Node,
  ) -> ResultAgalValue {
    self
      .0
      .borrow_mut()
      .define(stack, name, value, is_constant, node)
  }
  pub fn get_key(&self, name: &str) -> DefaultRefAgalValue {
    self
      .0
      .borrow()
      .clone()
      .variables
      .borrow_mut()
      .get(name)
      .unwrap()
      .clone()
  }
  pub fn get(&self, stack: RefStack, name: &str, node: &parser::Node) -> ResultAgalValue {
    self.0.borrow().get(stack, name, node)
  }
  pub fn _has(&self, name: &str) -> bool {
    self.0.borrow()._has(name)
  }
  pub fn resolve(&self, name: &str, node: &parser::Node) -> RefEnvironment {
    if !self._has(name) && self.has_parent() {
      let a = self.get_parent();
      return a.resolve(name, node);
    }
    return self.clone();
  }
  pub fn delete(&self, name: &str) -> bool {
    self.0.borrow().delete(name)
  }
  pub fn assign(
    &mut self,
    stack: RefStack,
    name: &str,
    value: DefaultRefAgalValue,
    node: &parser::Node,
  ) -> ResultAgalValue {
    self.0.borrow_mut().assign(stack, name, value, node)
  }
  pub fn is_constant(&self, name: &str) -> bool {
    self.0.borrow().is_constant(name)
  }
  pub fn is_keyword(&self, ref name: &str) -> bool {
    KEYWORDS.contains(name)
  }
}
