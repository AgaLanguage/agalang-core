use std::rc::Rc;

use super::{env::RefEnvironment, AgalValue, RefAgalValue, Stack};
use crate::{frontend, internal, runtime};

type EvalResult = Result<RefAgalValue, ()>;

fn code(filename: &str) -> Option<String> {
    let path = std::path::Path::new(filename);
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

pub fn full_eval(path: String, stack: &Stack, env: RefEnvironment) -> EvalResult {
    let contents = code(&path);
    if contents.is_none() {
        println!("Error al leer el archivo");
        return Err(());
    }

    let contents = contents.unwrap();
    eval(contents, path, stack, env)
}

pub fn eval(code: String, path: String, stack: &Stack, env: RefEnvironment) -> EvalResult {
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
    let value = runtime::interpreter(&program, stack, Rc::clone(&env));
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
