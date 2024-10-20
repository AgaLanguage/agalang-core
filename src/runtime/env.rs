use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{AgalBoolean, AgalThrow::Params, AgalValue, RefAgalValue, Stack};
use parser::{ast::Node, internal::ErrorNames};

pub type RefEnvironment = Rc<RefCell<Environment>>;

#[derive(Clone, PartialEq)]
pub struct Environment {
    in_class: bool,
    parent: Option<RefEnvironment>,
    variables: HashMap<String, RefAgalValue>,
    constants: HashSet<String>,
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
    SUPER_KEYWORD
];

impl Environment {
    pub fn get_global(&self) -> RefEnvironment {
        let mut env = self.clone().as_ref();
        while env.borrow().parent.is_some() {
            env = env.clone().borrow().parent.clone().unwrap();
        }
        env
    }
    pub fn as_ref(self) -> RefEnvironment {
        Rc::new(RefCell::new(self))
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
    pub fn crate_child(self, in_class: bool) -> Environment {
        Environment {
            in_class,
            parent: Some(self.as_ref()),
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
            return value.as_ref();
        }
        if self.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} ya ha sido declarada", name),
                stack: Box::new(stack.clone()),
            });
            return value.as_ref();
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
            return value.as_ref();
        }
        if !self.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} no ha sido declarada", name),
                stack: Box::new(Stack::get_default()),
            });
            return value.as_ref();
        }
        if self.constants.contains(name) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: "No se puede reasignar una constante".to_string(),
                stack: Box::new(Stack::get_default()),
            });
            return value.as_ref();
        }
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn set(&mut self, name: &str, value: RefAgalValue) -> RefAgalValue {
        self.variables.insert(name.to_string(), value.clone());
        value
    }
    pub fn get(&self, name: &str, node: &Node) -> RefAgalValue {
        let env = self.resolve(name, node);
        let env = env.borrow();
        if !env.has(name, node) {
            let value = AgalValue::Throw(Params {
                type_error: ErrorNames::EnviromentError,
                message: format!("La variable {} no ha sido declarada", name),
                stack: Box::new(Stack::get_default()),
            });
            return value.as_ref();
        }
        env.variables.get(name).unwrap().clone()
    }
    fn _has(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn has(&self, name: &str, node: &Node) -> bool {
        self.resolve(name, node)
            .borrow()
            .variables
            .contains_key(name)
    }
    fn resolve(&self, name: &str, node: &Node) -> RefEnvironment {
        if !self._has(name) && self.parent.is_some() {
            return self.parent.clone().unwrap().borrow().resolve(name, node);
        }
        return self.clone().as_ref();
    }
}
impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        for (name, value) in &self.variables {
            output.push_str(&format!("{}: {};\n", name, value.borrow()));
        }
        if self.parent.is_some() {
            output.push_str(&format!("{}", self.parent.clone().unwrap().borrow()));
        }
        write!(f, "{}", output)
    }
}
pub fn get_default() -> Environment {
    let mut env = Environment {
        in_class: false,
        parent: None,
        variables: HashMap::new(),
        constants: HashSet::new(),
    };
    env.variables.insert(
        TRUE_KEYWORD.to_string(),
        AgalValue::Boolean(AgalBoolean::new(true)).as_ref(),
    );
    env.variables.insert(
        FALSE_KEYWORD.to_string(),
        AgalValue::Boolean(AgalBoolean::new(false)).as_ref(),
    );
    env.variables
        .insert(NULL_KEYWORD.to_string(), AgalValue::Null.as_ref());
    env.variables
        .insert(NOTHING_KEYWORD.to_string(), AgalValue::Never.as_ref());
    env
}
