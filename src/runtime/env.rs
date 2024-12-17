use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{AgalBoolean, AgalThrow, AgalValuable, AgalValue, RefAgalValue, Stack};
use parser::{ast::Node, internal::ErrorNames};

pub type RefEnvironment<'a> = Rc<RefCell<Environment<'a>>>;

#[derive(Clone, PartialEq)]
pub struct Environment<'a> {
    in_class: bool,
    parent: Option<RefEnvironment<'a>>,
    variables: Rc<RefCell<HashMap<String, RefAgalValue<'a>>>>,
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

impl<'a> Environment<'a> {
    pub fn get_global(&self) -> RefEnvironment<'a> {
        let mut env = self.clone().as_ref();
        while env.borrow().parent.is_some() {
            env = env.clone().borrow().parent.clone().unwrap();
        }
        env
    }
    pub fn as_ref(self) -> RefEnvironment<'a> {
        Rc::new(RefCell::new(self))
    }
    pub fn get_this(&'a self, stack: &Stack, node: &Node) -> RefAgalValue<'a> {
        self.get(stack, THIS_KEYWORD, node)
    }
    pub fn use_private(self) -> bool {
        if self.in_class {
            true
        } else if let Some(p) = self.parent {
            p.borrow().clone().use_private()
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
    fn is_keyword(&self, ref name: &str) -> bool {
        KEYWORDS.contains(name)
    }
    pub fn define(
        &'a mut self,
        stack: &Stack,
        name: &String,
        value: RefAgalValue<'a>,
        is_constant: bool,
        node: &Node,
    ) -> RefAgalValue {
        if self.is_keyword(name) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede declarar una variable con el nombre de una palabra clave"
                    .to_string(),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        if self.has(name, node) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} ya ha sido declarada", name),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        if is_constant {
            self.constants.borrow_mut().insert(name.to_string());
        }
        self.variables
            .borrow_mut()
            .insert(name.to_string(), value.clone());
        value
    }
    pub fn assign(&'a mut self, stack: &Stack, name: &str, value: RefAgalValue<'a>, node: &Node) -> RefAgalValue<'a> {
        if self.is_keyword(name) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede reasignar una palabra clave".to_string(),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        if !self.has(name, node) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} ya ha sido declarada", name),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        if self.constants.borrow_mut().contains(name) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede reasignar una constante".to_string(),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        self.variables
            .borrow_mut()
            .insert(name.to_string(), value.clone());
        value
    }
    pub fn set(&'a mut self, name: &str, value: RefAgalValue<'a>) -> RefAgalValue<'a> {
        self.variables
            .borrow_mut()
            .insert(name.to_string(), value.clone());
        value
    }
    pub fn get(&self, stack: &Stack, name: &str, node: &Node) -> RefAgalValue<'a> {
        let _env = self.resolve(name, node);
        let env = _env.borrow_mut();
        if !env.has(name, node) {
            return AgalThrow::Params {
                type_error: ErrorNames::EnviromentError,
                message:  format!("La variable {} no ha sido declarada", name),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value();
        }
        let a = env.variables.borrow_mut().get(name).unwrap().clone();
        a
    }
    fn _has(&self, name: &str) -> bool {
        self.variables.borrow_mut().contains_key(name)
    }
    pub fn has(&self, name: &str, node: &Node) -> bool {
        self.resolve(name, node)
            .borrow()
            .variables
            .borrow_mut()
            .contains_key(name)
    }
    fn resolve(&self, name: &str, node: &Node) -> RefEnvironment<'a> {
        if !self._has(name) && self.parent.is_some() {
            let a = self.parent.clone().unwrap();
            return a.borrow().resolve(name, node);
        }
        return self.clone().as_ref();
    }
}
pub fn get_default<'a>() -> Environment<'a> {
    let mut env = Environment {
        in_class: false,
        parent: None,
        variables: Rc::new(RefCell::new(HashMap::new())),
        constants: Rc::new(RefCell::new(HashSet::new())),
    };
    env.variables.borrow_mut().insert(
        TRUE_KEYWORD.to_string(),
        AgalBoolean::new(true).to_ref_value(),
    );
    env.variables.borrow_mut().insert(
        FALSE_KEYWORD.to_string(),
        AgalBoolean::new(false).to_ref_value(),
    );
    env.variables
        .borrow_mut()
        .insert(NULL_KEYWORD.to_string(), AgalValue::Null.as_ref());
    env.variables
        .borrow_mut()
        .insert(NOTHING_KEYWORD.to_string(), AgalValue::Never.as_ref());
    env
}
