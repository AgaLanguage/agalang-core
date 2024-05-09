pub enum ErrorTypes {
  //FmtError(std::fmt::Error),
  IoError(std::io::Error),
  ErrorError(Box<dyn std::error::Error>),
  StringError(String),
}
pub enum ErrorNames {
  PathError,
  LexerError,
  SyntaxError,
  CustomError(String)
}
impl std::fmt::Display for ErrorNames {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ErrorNames::PathError => write!(f, "Error ruta"),
      ErrorNames::LexerError => write!(f, "Error léxico"),
      ErrorNames::SyntaxError => write!(f, "Error sintáctico"),
      ErrorNames::CustomError(s) => write!(f, "{s}"),
    }
  }
}

pub fn show_error(type_err: &ErrorNames, err: ErrorTypes) {
  let red_error = "\x1b[1m\x1b[91merror\x1b[39m:\x1b[0m";
  match err {
      //ErrorTypes::FmtError(e) => {
      //    eprintln!("{}: {}: {}", red_error, type_err, e);
      //}
      ErrorTypes::IoError(e) => {
          eprintln!("{red_error} {}: {}", type_err, e);
      }
      ErrorTypes::ErrorError(e) => {
          eprintln!("{red_error} {}: {}", type_err, e);
      }
      ErrorTypes::StringError(e) => {
          eprintln!("{red_error} {}: {}", type_err, e);
      }
  }
}
pub fn show_multiple_errors(type_err: ErrorNames, errs: Vec<ErrorTypes>) {
  println!("{}", errs.len());
  for err in errs {
      show_error(&type_err, err);
  };
}