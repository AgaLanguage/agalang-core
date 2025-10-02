#![allow(dead_code)]
pub enum ErrorTypes {
  //FmtError(std::fmt::Error),
  Io(std::io::Error),
  Error(Box<dyn std::error::Error>),
  String(String),
}
impl std::fmt::Display for ErrorTypes {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      //Self::FmtError(e) => write!(f,"{e}"),
      Self::Io(e) => write!(f, "{e}"),
      Self::Error(e) => write!(f, "{e}"),
      Self::String(e) => write!(f, "{e}"),
    }
  }
}
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorNames {
  None,
  PathError,
  LexerError,
  SyntaxError,
  CustomError(&'static str),
  EnvironmentError,
  MathError,
  TypeError,
}
impl std::fmt::Display for ErrorNames {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ErrorNames::None => write!(f, ""),
      ErrorNames::TypeError => write!(f, "Error de tipo"),
      ErrorNames::MathError => write!(f, "Error matemático"),
      ErrorNames::PathError => write!(f, "Error ruta"),
      ErrorNames::LexerError => write!(f, "Error léxico"),
      ErrorNames::SyntaxError => write!(f, "Error sintáctico"),
      ErrorNames::EnvironmentError => write!(f, "Error de entorno"),
      ErrorNames::CustomError(s) => write!(f, "{s}"),
    }
  }
}

const RED_ERROR: &str = "\x1b[1m\x1b[91merror\x1b[39m:\x1b[0m";

pub fn error_to_string(type_err: &ErrorNames, error: ErrorTypes) -> String {
  match type_err {
    ErrorNames::None => error.to_string(),
    _ => format!("{type_err}: {error}"),
  }
}
pub fn show_error(type_err: &ErrorNames, error: ErrorTypes) {
  let data = error_to_string(type_err, error);
  print_error(data);
}
pub fn show_multiple_errors(type_err: &ErrorNames, errors: Vec<ErrorTypes>) {
  for err in errors {
    show_error(type_err, err);
  }
}

pub fn print_error(data: String) {
  eprintln!("{RED_ERROR} {}", data);
}
