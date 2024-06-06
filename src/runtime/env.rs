use std::collections::{HashMap, HashSet};

use super::{AgalBoolean, AgalValue};
use crate::frontend::ast::Node;

#[derive(Clone)]
pub struct Enviroment {
    parent: Option<Box<Enviroment>>,
    variables: HashMap<String, AgalValue>,
    constants: HashSet<String>,
}
pub const TRUE_KEYWORD: &str = "cierto";
pub const FALSE_KEYWORD: &str = "falso";
pub const NULL_KEYWORD: &str = "nulo";
pub const NOTHING_KEYWORD: &str = "nada";
const KEYWORDS: [&str; 4] = [TRUE_KEYWORD, FALSE_KEYWORD, NULL_KEYWORD, NOTHING_KEYWORD];

impl Enviroment {
    pub fn crate_child(self) -> Enviroment {
        Enviroment {
            parent: Some(Box::new(self)),
            variables: HashMap::new(),
            constants: HashSet::new(),
        }
    }
    fn is_keyword(&self, ref name: &str) -> bool {
        KEYWORDS.contains(name)
    }
    pub fn define(
        &mut self,
        name: &String,
        value: AgalValue,
        is_constant: bool,
        node: &Node,
    ) -> AgalValue {
        if self.is_keyword(name) {
            panic!("No se puede declarar una variable con el nombre de una palabra clave");
        }
        if self.has(name, node) {
            panic!("La variable {} ya ha sido declarada", name);
        }
        if is_constant {
            self.constants.insert(name.to_string());
        }
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn assign(&mut self, name: &str, value: AgalValue, node: &Node) -> AgalValue {
        if self.is_keyword(name) {
            panic!("No se puede reasignar una palabra clave");
        }
        if !self.has(name, node) {
            panic!("La variable {} no ha sido declarada", name);
        }
        if self.constants.contains(name) {
            panic!("No se puede reasignar una constante");
        }
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn get(&self, name: &str, node: &Node) -> AgalValue {
        if self.is_keyword(name) {
            panic!("No se puede declarar una variable con el nombre de una palabra clave");
        }
        let env = self.resolve(name, node);
        if !env.has(name, node) {
            panic!("La variable {} no ha sido declarada", name);
        }
        env.variables.get(name).unwrap().clone()
    }
    fn _has(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn has(&self, name: &str, node: &Node) -> bool {
        self.resolve(name, node).variables.contains_key(name)
    }
    pub fn resolve(&self, name: &str, node: &Node) -> &Enviroment {
        if !self._has(name) && self.parent.is_some() {
            return self.parent.as_ref().unwrap().resolve(name, node);
        }
        return self;
    }
    pub fn inmut(&mut self) -> &Enviroment{
      self
    }
    pub fn as_ref(&self) -> &Enviroment {
        self
    }
}
impl std::fmt::Display for Enviroment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        for (name, value) in &self.variables {
            output.push_str(&format!("{}: {};\n", name, value));
        }
        if self.parent.is_some() {
            output.push_str(&format!("{}", self.parent.as_ref().unwrap()));
        }
        write!(f, "{}", output)
    }
}
pub fn get_default() -> Enviroment {
    let mut env = Enviroment {
        parent: None,
        variables: HashMap::new(),
        constants: HashSet::new(),
    };
    env.variables.insert(
        TRUE_KEYWORD.to_string(),
        AgalValue::Boolean(AgalBoolean::new(true)),
    );
    env.variables.insert(
        FALSE_KEYWORD.to_string(),
        AgalValue::Boolean(AgalBoolean::new(false)),
    );
    env.variables
        .insert(NULL_KEYWORD.to_string(), AgalValue::Null);
    env.variables
        .insert(NOTHING_KEYWORD.to_string(), AgalValue::Never);
    env
}
