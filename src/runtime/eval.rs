use std::{path::Path, rc::Rc};

use super::{env::RefEnvironment, AgalValue, RefAgalValue, Stack};
use parser as frontend;
use parser::internal;

use crate::{runtime, Modules, ToResult};

type EvalResult = Result<RefAgalValue, ()>;

fn code(path: &str) -> Option<String> {
    let contents = std::fs::read_to_string(path);
    match contents {
        Ok(contents) => Some(contents),
        Err(err) => {
            let ref type_err = internal::ErrorNames::PathError;
            let err = internal::ErrorTypes::IoError(err);
            internal::show_error(type_err, err);
            None
        }
    }
}

pub fn full_eval(
    path: &str,
    stack: &Stack,
    env: RefEnvironment,
    modules_manager: &Modules,
) -> EvalResult {
    if modules_manager.has(path) {
        return Ok(modules_manager.get(path));
    }
    let contents = code(&path);
    if contents.is_none() {
        return Err(());
    }

    let contents = contents.unwrap();
    let value = eval(contents, path, stack, env, &modules_manager.clone());
    if let Ok(value) = &value {
        modules_manager.add(path, value.clone());
    }
    value
}

fn eval(
    code: String,
    path: &str,
    stack: &Stack,
    env: RefEnvironment,
    modules_manager: &Modules,
) -> EvalResult {
    let program: frontend::ast::Node = {
        let mut parser = frontend::Parser::new(code, &path);
        parser.produce_ast()
    };
    if program.is_error() {
        let type_err = internal::errors::ErrorNames::SyntaxError;
        let node_err = program.get_error().unwrap();
        let err = frontend::node_error(&node_err);
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return Err(());
    }
    let env = env.borrow().clone().crate_child(false).as_ref();
    let value = runtime::interpreter(&program, stack, Rc::clone(&env), &modules_manager.clone());
    let result = Ok(value.clone());
    let value: &AgalValue = &value.borrow();
    if let AgalValue::Throw(err) = value {
        let error = err.get_error();
        let type_err = error.get_type_error();
        let err = error.to_error();
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return Err(());
    }
    result
}
