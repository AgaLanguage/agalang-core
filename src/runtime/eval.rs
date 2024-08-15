use std::{cell::RefCell, rc::Rc};

use super::{AgalValue, Enviroment, Stack};
use crate::{frontend, internal, runtime};

type EvalResult = Result<Rc<RefCell<Enviroment>>, ()>;

fn code(filename: &str) -> Option<String> {
    let contents = std::fs::read_to_string(filename);
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

pub fn full_eval(path: String, stack: &Stack, env: Enviroment) -> EvalResult {
    let contents = code(&path);
    if contents.is_none() {
        println!("Error al leer el archivo");
        return Err(());
    }

    let contents = contents.unwrap();
    eval(contents, path, stack, env)
}

pub fn eval(code: String, path: String, stack: &Stack, env: Enviroment) -> EvalResult {
    let program: frontend::ast::Node = {
        let mut parser = frontend::Parser::new(code, &path);
        parser.produce_ast()
    };
    println!("{}", program);
    if program.is_error() {
        let type_err = internal::errors::ErrorNames::SyntaxError;
        let node_err = program.get_error().unwrap();
        let err = frontend::node_error(&node_err);
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return Err(());
    }
    let env = Rc::new(RefCell::new(env.crate_child()));
    let value = runtime::interpreter(&program, stack, Rc::clone(&env));
    let value: &AgalValue = &value.borrow();
    if let AgalValue::Throw(err) = value {
        let error = err.get_error();
        let type_err = error.get_type_error();
        let err = error.to_error();
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return Err(());
    }
    Ok(env)
}
