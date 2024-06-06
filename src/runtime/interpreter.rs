use std::{cell::RefCell, rc::Rc};

use super::{AgalString, AgalValuable, AgalValue, Enviroment, Stack};
use crate::frontend::ast::{Node, NodeProperty, StringData};

pub fn interpreter(node: &Node, stack: &Stack, env: Rc<RefCell<Enviroment>>) -> AgalValue {
    let stack = stack.next(node);
    match node {
        Node::Program(program) => {
            let mut value = AgalValue::Never;
            for n in program.body.iter() {
                value = interpreter(n, &stack, Rc::clone(&env));
            }
            value
        }
        Node::String(str) => {
            let mut string = "".to_string();
            for s in str.value.iter() {
                match s {
                    StringData::Id(id) => {
                        let data = env.borrow_mut().get(id, node);
                        let env = env.borrow();
                        let env = env.as_ref();
                        let data = data.to_agal_string(Box::new(stack.clone()), env);
                        match data {
                            Ok(data) => string.push_str(&data.get_string()),
                            Err(err) => {
                                return AgalValue::Throw(err);
                            }
                        }
                    
                    },
                    StringData::Str(str) => string.push_str(str),
                }
            }
            AgalValue::String(AgalString::from_string(string))
        }
        Node::Array(list) => {
            let mut vec = vec![];
            for n in list.elements.iter() {
                match n {
                    NodeProperty::Indexable(value) => {
                        let data = interpreter(value, &stack, env.clone());
                        vec.push(data);
                    }
                    NodeProperty::Iterable(id) => {
                        let data = env.borrow_mut().get(&id.name, node);
                    }
                    _ => {}
                }
            }
            AgalValue::Array(vec)
        }
        Node::VarDecl(var) => match &var.value {
            Some(value) => {
                let value = interpreter(value, &stack, env.clone());
                env.borrow_mut().define(&var.name, value, var.is_const, node)
            }
            None => {
                env.borrow_mut().define(&var.name, AgalValue::Never, var.is_const, node)
            }
        },
        _ => AgalValue::Never,
    }
}
