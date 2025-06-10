use std::collections::VecDeque;
use std::path::Path;
use std::{collections::HashMap, process::ExitCode};

use crate::util::{OnError, OnSome};
use crate::{compiler::Compiler, interpreter::interpret};

mod compiler;
mod functions_names;
mod interpreter;
mod parser;
mod util;

use crate::compiler::binary::{Decode, Encode, StructTag};

const EXTENSION_COMPILE: &str = "agac";
const EXTENSION: &str = "aga";

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
    let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
    println!("{} {} <filename>", blue_usage, args.binary);
    return ExitCode::FAILURE;
  } else {
    args.file
  };
  let path = Path::new(&file_name);
  let (compiler, extension) = match compile(path) {
    Err(_) => return ExitCode::FAILURE,
    Ok(v) => v,
  };

  if args.action == Action::Compile && extension == EXTENSION {
    let code = compiler.encode();
    let default_name = path.file_stem().on_some_option(|v| v.to_str()).unwrap();
    let name = args
      .flags
      .get("nombre")
      .or_else(|| args.flags.get("name"))
      .on_some_option(|v| {
        if let FlagValue::String(s) = v {
          Some(s.as_str())
        } else {
          None
        }
      })
      .unwrap_or(default_name);
    match code {
      Ok(code) => {
        let _ = std::fs::write(format!("{name}.{EXTENSION_COMPILE}"), &code);
      }
      Err(e) => {
        eprintln!("{e}");
      }
    };
  }
  let run_flag = args.flags.contains_key("r")
    || args.flags.contains_key("run")
    || args.flags.contains_key("e")
    || args.flags.contains_key("ejecutar");
  if args.action == Action::Run || run_flag {
    return match interpret(compiler) {
      Err(_) => ExitCode::FAILURE,
      _ => ExitCode::SUCCESS,
    };
  }
  return ExitCode::SUCCESS;
}

fn read_code(path: &Path) -> Option<String> {
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
fn read_bin(path: &Path) -> Option<Vec<u8>> {
  let contents = std::fs::read(path);
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
fn compile(path: &Path) -> Result<(Compiler, &str), &str> {
  match path.extension().on_some_option(|v| v.to_str()) {
    Some(EXTENSION) => {
      let file = read_code(path).on_error(|_| "No se pudo leer el archivo")?;
      let ast = parser::Parser::new(&file, path.file_name().unwrap().to_str().unwrap())
        .produce_ast()
        .on_error(|e| {
          parser::print_error(parser::error_to_string(
            &parser::ErrorNames::SyntaxError,
            parser::node_error(&e, &file),
          ));
          ""
        })?;
      Ok((Compiler::from(&ast), EXTENSION))
    }
    Some(EXTENSION_COMPILE) => {
      let bin = read_bin(path).on_error(|_| "")?;
      let compiler = Compiler::decode(&mut VecDeque::from(bin)).on_error(|_| "")?;
      Ok((compiler, EXTENSION_COMPILE))
    }
    _ => Err("Se esperaba un archivo con extension valida"),
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
      _ => false,
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
  _args: Vec<String>,
}

impl Arguments {
  fn init() -> Self {
    let mut cmd_args = std::env::args().skip(1).peekable(); // skip the binary name
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
        let next = cmd_args.peek(); // peek
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
      _args: args,
    }
  }
}
