#![allow(warnings)]
mod colors;
mod libraries;
mod path;
mod runtime;

use std::{
  cell::RefCell, collections::HashMap, process::ExitCode, rc::Rc, thread::sleep, time::Duration,
};

use runtime::values::DefaultRefAgalValue;

trait ToResult<T> {
  fn to_result(self) -> Result<T, ()>;
}

impl<T> ToResult<T> for Option<T> {
  fn to_result(self) -> Result<T, ()> {
    if let Some(v) = self {
      Ok(v)
    } else {
      Err(())
    }
  }
}
trait OnError<T, E> {
  fn on_error(self, error: E) -> Result<T, E>;
}
impl<T, E> OnError<T, E> for Option<T> {
  fn on_error(self, error: E) -> Result<T, E> {
    match self {
      Some(v) => Ok(v),
      None => Err(error),
    }
  }
}
impl<T, E, V> OnError<T, E> for Result<T, V> {
  fn on_error(self, error: E) -> Result<T, E> {
    match self {
      Ok(v) => Ok(v),
      Err(e) => Err(error),
    }
  }
}

#[tokio::main]
async fn main() -> ExitCode {
  let modules_manager = libraries::RefModules::new();
  let filename = file();
  if filename.is_none() {
    return ExitCode::FAILURE;
  }
  let filename = filename.unwrap();
  let stack = runtime::RefStack::get_default();

  let program = runtime::full_eval(filename, stack, modules_manager).await;
  if program.is_none() {
    return ExitCode::FAILURE;
  }
  return ExitCode::SUCCESS;
}

fn file() -> Option<String> {
  let mut args: Vec<String> = std::env::args().collect();
  args.push("./file.agal".to_string());
  let args = args;
  if args.len() < 2 {
    let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
    println!("{} {} <filename>", blue_usage, args[0]);
    return None;
  }
  Some(args[1].to_string())
}
