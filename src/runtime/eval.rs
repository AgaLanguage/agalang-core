use std::{borrow::Borrow, path::Path, rc::Rc};

use parser::{self as frontend, util::RefValue};
use parser::internal;

use crate::{
  runtime::{self, env::RefEnvironment},
  Modules, ToResult,
};

use super::{
  stack::Stack,
  values::{internal::AgalInternal, AgalValue, DefaultRefAgalValue},
};

type EvalResult = Result<DefaultRefAgalValue, ()>;

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

pub async fn full_eval(
  path: &str,
  stack: &Stack,
  env: RefEnvironment,
  modules: RefValue<Modules>,
) -> EvalResult {
  let modules_manager = &*modules.as_ref().borrow();
  if modules_manager.has(path) {
    return Ok(modules_manager.get(path));
  }
  let contents = code(&path).to_result()?;

  let value = eval(contents, path, stack, env, &modules_manager.clone()).await?;
  modules_manager.add(path, value.clone());
  Ok(value)
}

async fn eval(
  code: String,
  path: &str,
  stack: &Stack,
  env: RefEnvironment,
  modules_manager: &Modules,
) -> EvalResult {
  let program = {
    let mut parser = frontend::Parser::new(code, &path);
    parser.produce_ast()
  };
  let program = match program {
    Ok(value) => value,
    Err(err) => {
      let type_err = internal::errors::ErrorNames::SyntaxError;
      let err = internal::ErrorTypes::StringError(err.message);
      let data = internal::errors::error_to_string(&type_err, err);
      internal::print_error(data);
      return Err(());
    }
  };
  let env = env.crate_child(false);
  let value = runtime::interpreter::interpreter(
    program.to_box(),
    stack.clone().to_ref(),
    env.clone(),
    modules_manager.clone().to_ref(),
  )
  .await;
  let value = &*value.borrow();
  match value {
    Err(throw) =>{
      let (type_err, err) = throw.get_data();
      let data = internal::errors::error_to_string(&type_err, err);
      internal::print_error(data);
      Err(())
    }
    Ok(value) => {
      Ok(value.clone())
    }
  }
}
