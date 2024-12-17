use std::{path::Path, rc::Rc};

use parser as frontend;
use parser::internal;

use crate::{
    runtime::{self, env::RefEnvironment, AgalInternal, AgalValue, RefAgalValue, Stack},
    Modules, ToResult,
};

type EvalResult<'a> = Result<RefAgalValue<'a>, ()>;

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

pub async fn full_eval<'a>(
    path: &str,
    stack: &Stack,
    env: RefEnvironment<'a>,
    modules_manager: &Modules<'a>,
) -> EvalResult<'a> {
    if modules_manager.has(path) {
        return Ok(modules_manager.get(path));
    }
    let contents = code(&path);
    if contents.is_none() {
        return Err(());
    }

    let contents = contents.unwrap();
    let value = eval(contents, path, stack, env, &modules_manager.clone()).await?;
    modules_manager.add(path, value.clone());
    Ok(value)
}

async fn eval<'a>(
    code: String,
    path: &str,
    stack: &Stack,
    env: RefEnvironment<'a>,
    modules_manager: &Modules<'a>,
) -> EvalResult<'a> {
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
    let value = runtime::interpreter(program.to_box(), stack.clone().to_ref(), env.clone(), modules_manager.clone().to_ref()).await.await;
    let result = Ok(value.clone());
    let value: &AgalValue = &value.borrow();
    if let AgalValue::Internal(AgalInternal::Throw(err)) = value {
        let error = err.get_error();
        let type_err = error.get_type_error();
        let err = error.to_error();
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return Err(());
    }
    result
}
