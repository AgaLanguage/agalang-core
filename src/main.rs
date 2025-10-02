use std::collections::VecDeque;
use std::path::Path;
use std::{collections::HashMap, process::ExitCode};

pub use crate::util::{MultiRefHash, OnError, OnSome};
use crate::{compiler::Compiler, interpreter::interpret};

mod agal_parser;
mod compiler;
mod functions_names;
mod interpreter;
mod tokens;
mod util;

use crate::compiler::binary::{Decode, Encode, StructTag};

const EXTENSION_BYTECODE: &str = "agab";
const EXTENSION: &str = "aga";

fn main() -> ExitCode {
  let args = Arguments::init();

  if args.action == Action::Help {
    println!("Ayudando...");
    return ExitCode::SUCCESS;
  }
  if let Action::Unknown(action) = args.action {
    eprintln!("Acción '{action}' desconocida");
    return ExitCode::FAILURE;
  }

  let file_name = if args.file.is_empty() {
    let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
    eprintln!("{} {} <filename>", blue_usage, args.binary);
    return ExitCode::FAILURE;
  } else {
    args.file.clone()
  };
  let path = Path::new(&file_name);
  if args.action == Action::SyntaxisTokens
    && path
      .extension()
      .unwrap_or_default()
      .to_str()
      .unwrap_or_default()
      == EXTENSION
  {
    let code = read_code(path).on_some_option(|file| {
      agal_parser::Parser::new(&file, path)
        .produce_ast()
        .on_error(|e| {
          if !e.message.is_empty() {
            agal_parser::print_error(agal_parser::error_to_string(
              &agal_parser::ErrorNames::SyntaxError,
              agal_parser::node_error(&e, &file),
            ));
          }
          ""
        })
        .ok()
    });
    return match code {
      Some(node) => {
        tokens::print_tokens(node);
        ExitCode::SUCCESS
      }
      None => ExitCode::FAILURE,
    };
  }
  let (compiler, extension) = match compile(path) {
    Err(e) => {
      if !e.is_empty() {
        eprintln!("{e}");
      }
      return ExitCode::FAILURE;
    }
    Ok(v) => v,
  };

  if args.action == Action::Compile && extension == EXTENSION {
    let code = compiler.encode();
    let default_name = path.file_stem().on_some_option(|v| v.to_str()).unwrap();
    let name = args.get_string(&FlagName::Name);
    let name = if name.is_empty() { default_name } else { name };
    let code = match code {
      Ok(code) => code,
      Err(e) => {
        eprintln!("{e}");
        return ExitCode::FAILURE;
      }
    };
    let _ = std::fs::write(format!("{name}.{EXTENSION_BYTECODE}"), &code);
  }
  if args.action == Action::Run || args.get_bool(&FlagName::Name) {
    return match interpret(compiler) {
      Err(_) => ExitCode::FAILURE,
      _ => ExitCode::SUCCESS,
    };
  }
  ExitCode::SUCCESS
}

fn read_code(path: &Path) -> Option<String> {
  let contents = std::fs::read_to_string(path);
  match contents {
    Ok(contents) => Some(contents),
    Err(err) => {
      let type_err = agal_parser::ErrorNames::PathError;
      let err = agal_parser::ErrorTypes::Io(err);
      agal_parser::show_error(&type_err, err);
      None
    }
  }
}
fn read_bin(path: &Path) -> Option<Vec<u8>> {
  let contents = std::fs::read(path);
  match contents {
    Ok(contents) => Some(contents),
    Err(err) => {
      let type_err = agal_parser::ErrorNames::PathError;
      let err = agal_parser::ErrorTypes::Io(err);
      agal_parser::show_error(&type_err, err);
      None
    }
  }
}
fn compile_bytecode(vec: Vec<u8>) -> Result<Compiler, String> {
  Compiler::decode(&mut VecDeque::from(vec))
}
fn compile(path: &Path) -> Result<(Compiler, &str), String> {
  match path.extension().on_some_option(|v| v.to_str()) {
    Some(EXTENSION) => {
      let file = read_code(path).on_error(|_| "No se pudo leer el archivo")?;
      let ast = agal_parser::Parser::new(&file, path)
        .produce_ast()
        .on_error(|e| {
          if !e.message.is_empty() {
            agal_parser::print_error(agal_parser::error_to_string(
              &agal_parser::ErrorNames::SyntaxError,
              agal_parser::node_error(&e, &file),
            ));
          }
          ""
        })?;
      Ok(((&ast).try_into()?, EXTENSION))
    }
    Some(EXTENSION_BYTECODE) => Ok((
      compile_bytecode(read_bin(path).on_error(|_| "")?)?,
      EXTENSION_BYTECODE,
    )),
    _ => Err("Se esperaba un archivo con extension valida".to_string()),
  }
}

#[derive(PartialEq, Eq, Hash)]
enum FlagName {
  Run,
  Name,
  Compress,
  Help,
  None,
}
impl From<String> for FlagName {
  fn from(value: String) -> Self {
    match value.as_str() {
      "ejecutar" | "run" | "e" | "r" => Self::Run,
      "comprimir" | "compress" | "c" => Self::Compress,
      "ayuda" | "help" | "a" | "h" => Self::Help,
      "nombre" | "name" | "n" => Self::Name,
      _ => Self::None,
    }
  }
}

enum FlagValue {
  String(String),
  Boolean(bool),
}
#[derive(PartialEq, Eq)]
enum Action {
  Compile,
  Run,
  Help,
  SyntaxisTokens,
  Unknown(String),
}
impl Action {
  pub fn is_unknown(&self) -> bool {
    matches!(self, Self::Unknown(_))
  }
}
impl From<String> for Action {
  fn from(value: String) -> Self {
    match value.to_lowercase().as_str() {
      "ejecutar" | "run" | "e" | "r" => Action::Run,
      "compilar" | "compile" | "c" => Action::Compile,
      "ayuda" | "help" | "a" | "h" => Action::Help,
      "tokens" => Action::SyntaxisTokens,
      _ => Action::Unknown(value),
    }
  }
}
struct Arguments {
  binary: String,
  flags: HashMap<FlagName, FlagValue>,
  action: Action,
  file: String,
  _args: Vec<String>,
}
fn remove_first_and_last(s: &str) -> String {
  let mut chars = s.chars();
  chars.next();
  chars.next_back();
  chars.collect()
}
impl Arguments {
  fn init() -> Self {
    let mut cmd_args = std::env::args().skip(1).peekable(); // skip the binary name
    let binary = std::env::args().next().unwrap_or_default();

    let mut flags = HashMap::new();
    let mut action = Action::Unknown(String::new());
    let mut file = String::new();
    let mut args = vec![];

    let mut on_string_arg = false;
    let mut string_arg = String::new();

    while let Some(arg) = cmd_args.next() {
      if !file.is_empty() {
        args.push(arg);
        continue;
      }
      if on_string_arg {
        if string_arg.ends_with('"') {
          on_string_arg = false;
        }
        string_arg.push_str(&arg);
      }
      let arg = if string_arg.is_empty() {
        arg
      } else {
        let data = remove_first_and_last(&string_arg);
        string_arg.clear();
        data
      };
      if arg.starts_with('"') {
        string_arg.push_str(&arg);
        on_string_arg = true;
      } else if arg.starts_with("--") {
        let key: FlagName = arg.trim_start_matches("--").to_string().into();
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
        flags.insert(key.into(), FlagValue::Boolean(true));
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
  fn get_bool(&self, key: &FlagName) -> bool {
    match self.flags.get(key) {
      Some(FlagValue::Boolean(b)) => *b,
      Some(FlagValue::String(s)) => {
        s.to_uppercase()
          .as_str()
          .chars()
          .collect::<Vec<char>>()
          .first()
          != Some(&'N')
      }
      _ => false,
    }
  }
  fn get_string(&self, key: &FlagName) -> &str {
    match self.flags.get(key) {
      Some(FlagValue::String(s)) => s,
      _ => "",
    }
  }
}

pub trait ToJSON {
  fn to_json(&self) -> String;
}
impl ToJSON for String {
  fn to_json(&self) -> String {
    format!(
      "\"{}\"",
      self
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
    )
    .replace('\r', "\\r")
    .replace('\t', "\\t")
    .replace('\0', "\\0")
  }
}
impl<T> ToJSON for Option<T>
where
  T: ToJSON,
{
  fn to_json(&self) -> String {
    match self {
      None => "null".to_string(),
      Some(t) => t.to_json(),
    }
  }
}
impl<T> ToJSON for Vec<T>
where
  T: ToJSON,
{
  fn to_json(&self) -> String {
    let mut data = String::new();
    data.push('[');
    let mut is_first = true;
    for item in self {
      if !is_first {
        data.push(',');
      }
      data.push_str(&item.to_json());
      is_first = false;
    }
    data.push(']');
    data
  }
}
impl<K, T> ToJSON for HashMap<K, T>
where
  T: ToJSON,
  K: ToString,
{
  fn to_json(&self) -> String {
    let mut json = String::new();
    json.push('{');
    let mut is_first = true;
    for (key, value) in self.iter() {
      if is_first {
        is_first = false;
      } else {
        json.push(',');
      }
      json.push_str(&key.to_string().to_json());
      json.push(':');
      json.push_str(&value.to_json());
    }
    json.push('}');
    json
  }
}
