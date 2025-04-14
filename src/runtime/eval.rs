use std::path::Path;

use crate::{
  libraries, parser,
  runtime::{self, env::RefEnvironment},
};

use super::{async_interpreter, interpreter};
use super::{
  stack::RefStack,
  values::{internal::AgalInternal, AgalValue, DefaultRefAgalValue},
};

type EvalResult = Option<DefaultRefAgalValue>;

fn code(path: &str) -> Option<String> {
  let contents = std::fs::read_to_string(path);
  match contents {
    Ok(contents) => Some(contents),
    Err(err) => {
      let ref type_err = parser::ErrorNames::PathError;
      let err = parser::ErrorTypes::IoError(err);
      parser::show_error(type_err, err);
      None
    }
  }
}

pub fn full_eval(
  path: String,
  stack: RefStack,
  modules_manager: libraries::RefModules,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = EvalResult> + Send>> {
  Box::pin(async move {
    if modules_manager.has(&path) {
      return modules_manager.try_get(&path);
    }
    let contents = code(&path)?;

    let value = eval(contents, &path, stack, modules_manager.clone()).await?;
    modules_manager.add(&path, value.clone());
    Some(value)
  })
}

async fn eval(
  code: String,
  path: &str,
  stack: RefStack,
  modules_manager: libraries::RefModules,
) -> EvalResult {
  let program = parser::Parser::new(code, &path).produce_ast();
  let program = match program {
    Ok(value) => value,
    Err(err) => {
      let type_err = parser::ErrorNames::SyntaxError;
      let data = parser::error_to_string(&type_err, parser::node_error(&err));
      parser::print_error(data);
      return None;
    }
  };
  let box_node = program.to_box();
  let new_stack = stack.crate_child(false, box_node.clone());
  let value = async_interpreter(box_node, new_stack, modules_manager).await;
  match value {
    Err(throw) => {
      let (type_err, err) = throw.get_data();
      let data = parser::error_to_string(&type_err, err);
      parser::print_error(data);
      None
    }
    Ok(value) => Some(value.clone()),
  }
}
