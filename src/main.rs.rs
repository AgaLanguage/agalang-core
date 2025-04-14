#![allow(warnings)]
mod functions_names;
mod libraries;
mod parser;
mod path;
mod runtime;
mod util;
mod bytecode;

use std::{collections::HashMap, process::ExitCode, thread::sleep, time::Duration};

use runtime::values::DefaultRefAgalValue;
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
