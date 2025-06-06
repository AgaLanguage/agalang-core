use std::{collections::HashMap, process::ExitCode};

use crate::{compiler::Compiler, interpreter::interpret};

mod compiler;
mod functions_names;
mod interpreter;
mod parser;
mod util;

fn main() -> ExitCode {
  let args = Arguments::init();

  if args.action == Action::Help {
    println!("Ayudando...");
    return ExitCode::SUCCESS;
  }
  if let Action::Unknown(action) = args.action {
    println!("Acción '{action}' desconocida");
    return ExitCode::FAILURE;
  }

  let file_name = if args.file.is_empty() {
    return ExitCode::FAILURE;
  } else {
    args.file
  };

  let file = match code(&file_name) {
    None => return ExitCode::FAILURE,
    Some(value) => value,
  };

  let ast = match parser::Parser::new(&file, &file_name).produce_ast() {
    Err(a) => {
      parser::print_error(parser::error_to_string(
        &parser::ErrorNames::SyntaxError,
        parser::node_error(&a, &file),
      ));
      return ExitCode::FAILURE;
    }
    Ok(value) => value,
  };

  let compile_code = Compiler::from(&ast);

  if args.action == Action::Run {
    return match interpret(compile_code) {
      Err(_) => ExitCode::FAILURE,
      _ => ExitCode::SUCCESS,
    };
  }
  println!("caracteristica no implementada");
  return ExitCode::SUCCESS;
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

enum FlagValue {
  String(String),
  Boolean(bool),
  Default,
}
#[derive(PartialEq, Eq)]
enum Action {
  Compile,
  Run,
  Help,
  Unknown(String),
}
impl Action {
  pub fn is_unknown(&self) -> bool {
    match self {
      Self::Unknown(_) => true,
      _ => false
    }
  }
}
impl From<String> for Action {
  fn from(value: String) -> Self {
    match value.to_lowercase().as_str() {
      "ejecutar" | "run" | "e" | "r" => Action::Run,
      "compilar" | "compile" | "c" => Action::Compile,
      "ayuda" | "help" | "a" | "h" => Action::Compile,
      _ => Action::Unknown(value),
    }
  }
}
struct Arguments {
  binary: String,
  flags: HashMap<String, FlagValue>,
  action: Action,
  file: String,
  args: Vec<String>,
}

impl Arguments {
  fn init() -> Self {
    let mut cmd_args = std::env::args().skip(1); // skip the binary name
    let binary = std::env::args().next().unwrap_or_default();

    let mut flags = HashMap::new();
    let mut action = Action::Unknown(String::new());
    let mut file = String::new();
    let mut args = vec![];

    while let Some(arg) = cmd_args.next() {
      if !file.is_empty() {
        args.push(arg);
        continue;
      }
      if arg.starts_with("--") {
        let key = arg.trim_start_matches("--").to_string();
        let next = cmd_args.next(); // peek
        if let Some(v) = next {
          if !v.starts_with('-') {
            flags.insert(key, FlagValue::String(cmd_args.next().unwrap()));
          } else {
            flags.insert(key, FlagValue::Boolean(true));
          }
        } else {
          flags.insert(key, FlagValue::Boolean(true));
        }
      } else if arg.starts_with("-") {
        let key = arg.trim_start_matches("-").to_string();
        flags.insert(key, FlagValue::Boolean(true));
      } else if action.is_unknown() {
        action = arg.into();
      } else {
        file = arg;
      }
    }
    Self {
      flags,
      binary,
      action,
      file,
      args,
    }
  }
}

fn file() -> Option<String> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
    println!("{} {} <filename>", blue_usage, args[0]);
    return None;
  }
  Some(args[1].to_string())
}
