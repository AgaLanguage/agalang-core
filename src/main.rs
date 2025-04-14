#![allow(warnings)]
use std::process::ExitCode;

mod bytecode;
mod parser;
mod util;

fn main() -> ExitCode {
  let file_name = match file() {
    None => return ExitCode::FAILURE,
    Some(value) => value,
  };
  let file = match code(&file_name) {
    None => return ExitCode::FAILURE,
    Some(value) => value,
  };

  let ast = match parser::Parser::new(file, &file_name).produce_ast() {
    Err(a) => {eprintln!("{a:?}");return ExitCode::FAILURE},
    Ok(value) => value,
  };

  match bytecode::main(&ast) {
    Err(e) => eprintln!("{e}"),
    _=>{}
  };
  ExitCode::SUCCESS
}
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

fn file() -> Option<String> {
  let mut args: Vec<String> = std::env::args().collect();
  println!("{args:?}");
  if args.len() < 2 {
    let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
    println!("{} {} <filename>", blue_usage, args[0]);
    return None;
  }
  Some(args[1].to_string())
}
