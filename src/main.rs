use std::collections::VecDeque;
use std::io::{Read as _, Write as _};
use std::path::Path;
use std::{collections::HashMap, process::ExitCode};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::util::{OnError, OnSome};
use crate::{compiler::Compiler, interpreter::interpret};

mod compiler;
mod functions_names;
mod interpreter;
mod parser;
mod util;

use crate::compiler::binary::{Decode, Encode, StructTag};

const EXTENSION_COMPRESS: &str = "agac";
const EXTENSION_BYTECODE: &str = "agab";
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
    args.file.clone()
  };
  let path = Path::new(&file_name);
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
    let (bin, bin_extension) = match code {
      Ok(code) => {
        if args.get_bool(&FlagName::Compress) {
          match compress_bytes(&code) {
            Err(e) => {
              eprintln!("{e}");
              return ExitCode::FAILURE;
            }
            Ok(bin) => (bin, EXTENSION_COMPRESS),
          }
        } else {
          (code, EXTENSION_BYTECODE)
        }
      }
      Err(e) => {
        eprintln!("{e}");
        return ExitCode::FAILURE;
      }
    };
    let _ = std::fs::write(format!("{name}.{bin_extension}"), &bin);
  }
  if args.action == Action::Run || args.get_bool(&FlagName::Name) {
    return match interpret(compiler) {
      Err(_) => ExitCode::FAILURE,
      _ => ExitCode::SUCCESS,
    };
  }
  return ExitCode::SUCCESS;
}

fn compress_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
  let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
  encoder.write_all(data)?; // Escribe los bytes al encoder
  let compressed_data = encoder.finish()?; // Termina la compresión y obtiene el Vec<u8>
  Ok(compressed_data)
}

// Descomprime bytes gzip y devuelve un Vec<u8>
fn decompress_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
  let mut decoder = GzDecoder::new(data);
  let mut decompressed = Vec::new();
  decoder.read_to_end(&mut decompressed)?; // Lee todo el contenido descomprimido
  Ok(decompressed)
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
fn compile_bytecode(vec: Vec<u8>) -> Result<Compiler, String> {
  Compiler::decode(&mut VecDeque::from(vec))
}
fn compile(path: &Path) -> Result<(Compiler, &str), String> {
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
    Some(EXTENSION_COMPRESS) => {
      let bin = read_bin(path).on_error(|_| "")?;
      match decompress_bytes(&bin) {
        Err(_) => Err("Error de descompresion".to_string()),
        Ok(uncompress) => Ok((compile_bytecode(uncompress)?, EXTENSION_COMPRESS)),
      }
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
      "ayuda" | "help" | "a" | "h" => Action::Help,
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
