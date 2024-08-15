use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use super::{AgalBoolean, AgalThrow::Params, AgalValue, Stack};
use crate::{frontend::ast::Node, internal::ErrorNames};

type RefAgalValue = Rc<RefCell<AgalValue>>;

#[derive(Clone, PartialEq)]
pub struct Enviroment {
    parent: Option<Box<Enviroment>>,
    variables: HashMap<String, RefAgalValue>,
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
        stack: &Stack,
        name: &String,
        value: RefAgalValue,
        is_constant: bool,
        node: &Node,
    ) -> RefAgalValue {
        if self.is_keyword(name) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede declarar una variable con el nombre de una palabra clave"
                    .to_string(),
                stack: Box::new(stack.clone()),
            });
            return Rc::new(RefCell::new(value));
        }
        if self.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} ya ha sido declarada", name),
                stack: Box::new(stack.clone()),
            });
            return Rc::new(RefCell::new(value));
        }
        if is_constant {
            self.constants.insert(name.to_string());
        }
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn assign(&mut self, name: &str, value: RefAgalValue, node: &Node) -> RefAgalValue {
        if self.is_keyword(name) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede reasignar una palabra clave".to_string(),
                stack: Box::new(Stack::get_default()),
            });
            return Rc::new(RefCell::new(value));
        }
        if !self.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} no ha sido declarada", name),
                stack: Box::new(Stack::get_default()),
            });
            return Rc::new(RefCell::new(value));
        }
        if self.constants.contains(name) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede reasignar una constante".to_string(),
                stack: Box::new(Stack::get_default()),
            });
            return Rc::new(RefCell::new(value));
        }
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn get(&self, name: &str, node: &Node) -> RefAgalValue {
        let env = self.resolve(name, node);
        if !env.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} no ha sido declarada", name),
                stack: Box::new(Stack::get_default()),
            });
            return Rc::new(RefCell::new(value));
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
    pub fn as_ref(&self) -> &Enviroment {
        self
    }
}
impl std::fmt::Display for Enviroment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        for (name, value) in &self.variables {
            output.push_str(&format!("{}: {};\n", name, value.borrow()));
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
        Rc::new(RefCell::new(AgalValue::Boolean(AgalBoolean::new(true)))),
    );
    env.variables.insert(
        FALSE_KEYWORD.to_string(),
        Rc::new(RefCell::new(AgalValue::Boolean(AgalBoolean::new(false)))),
    );
    env.variables
        .insert(NULL_KEYWORD.to_string(), 
        Rc::new(RefCell::new(AgalValue::Null)));
    env.variables
        .insert(NOTHING_KEYWORD.to_string(), Rc::new(RefCell::new(AgalValue::Never)));
    env
}
