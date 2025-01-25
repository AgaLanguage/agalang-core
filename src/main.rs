#![allow(warnings)]
mod libraries;
mod path;
mod runtime;

use std::{
  cell::RefCell, collections::HashMap, process::ExitCode, rc::Rc, thread::sleep, time::Duration,
};

use parser::util::RefValue;
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

#[derive(Clone)]
struct Modules {
  map: Rc<RefCell<HashMap<String, DefaultRefAgalValue>>>,
}
impl Modules {
  fn new() -> Self {
    Modules {
      map: Rc::new(RefCell::new(HashMap::new())),
    }
  }
  fn has(&self, key: &str) -> bool {
    self.map.borrow().contains_key(key)
  }
  fn get(&self, key: &str) -> DefaultRefAgalValue {
    let v = self.map.borrow();
    v.get(key).unwrap().clone()
  }
  fn add(&self, key: &str, value: DefaultRefAgalValue) {
    if self.has(key) {
      return;
    }
    let mut v = self.map.borrow_mut();
    v.insert(key.to_string(), value);
  }
  fn to_ref(self) -> RefValue<Self> {
    Rc::new(RefCell::new(self))
  }
}

#[tokio::main]
async fn main() -> ExitCode {
  let modules_manager = Modules::new();
  let filename = file();
  if filename.is_none() {
    return ExitCode::FAILURE;
  }
  let filename = filename.unwrap();
  let ref stack = runtime::Stack::get_default();
  let global_env = runtime::Environment::get_default().as_ref();

  let program = runtime::full_eval(&filename, stack, global_env, modules_manager.to_ref()).await;
  if program.is_err() {
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
