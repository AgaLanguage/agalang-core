use std::collections::{HashMap, HashSet};

use crate::frontend::ast::Node;

#[derive(Clone, Copy)]
struct AgalValue;
#[derive(Clone)]
pub struct Enviroment {
  parent: Option<Box<Enviroment>>,
  variables: HashMap<String, AgalValue>,
  constants: HashSet<String>,
}

const KEYWORDS: [&str;4] = ["cierto", "falso", "nulo", "nada"];

impl Enviroment {
  pub fn new() -> Enviroment {
    Enviroment {
      parent: None,
      variables: HashMap::new(),
      constants: HashSet::new(),
    }
  }
  pub fn crate_child(&mut self) -> Enviroment {
    let mut child = Enviroment::new();
    child.parent = Some(Box::new(self.clone()));
    child
  }
  fn is_keyword(&self, name: &str) -> bool {
    KEYWORDS.contains(&name)
  }
  pub fn define(&mut self, name: &str, value: AgalValue, is_constant: bool, _node: &Node, internal: bool) -> AgalValue {
    if self.is_keyword(name) && !internal {
      panic!("No se puede declarar una variable con el nombre de una palabra clave");
    }
    if self.has(name) {
      panic!("La variable {} ya ha sido declarada", name);
    }
    if is_constant {
      self.constants.insert(name.to_string());
    }
    self.variables.insert(name.to_string(), value);
    value
  }
  pub fn assign(&mut self, name: &str, value: AgalValue, _node: &Node) -> AgalValue {
    if self.is_keyword(name) {
      panic!("No se puede declarar una variable con el nombre de una palabra clave");
    }
    if !self.has(name) {
      panic!("La variable {} no ha sido declarada", name);
    }
    if self.constants.contains(name) {
      panic!("No se puede reasignar una constante");
    }
    self.variables.insert(name.to_string(), value);
    value
  }
  pub fn get(&self, name: &str, node: &Node) -> AgalValue {
    if self.is_keyword(name) {
      panic!("No se puede declarar una variable con el nombre de una palabra clave");
    }
    let env = self.resolve(name, node);
    if !env.has(name) {
      panic!("La variable {} no ha sido declarada", name);
    }
    env.variables.get(name).unwrap().clone()
  }
  pub fn has(&self, name: &str) -> bool {
    self.variables.contains_key(name)
  }
  pub fn resolve(&self, name: &str, node: &Node) -> &Enviroment {
    if self.has(name) {
      return self;
    }
    if self.parent.is_some() {
      return self.parent.as_ref().unwrap().resolve(name, node)
    }
    return self;
  }
}