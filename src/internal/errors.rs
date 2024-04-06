pub enum ErrorTypes {
  //FmtError(std::fmt::Error),
  IoError(std::io::Error),
  ErrorError(Box<dyn std::error::Error>),
  StringError(String),
}
pub enum ErrorNames {
  PathError,
  LexerError,
  ParserError,
  CustomError(String)
}
impl std::fmt::Display for ErrorNames {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      ErrorNames::PathError => write!(f, "Error ruta"),
      ErrorNames::LexerError => write!(f, "Error léxico"),
      ErrorNames::ParserError => write!(f, "Error de análisis"),
      ErrorNames::CustomError(s) => write!(f, "{s}"),
    }
  }
}

fn show_error(type_err: &ErrorNames, err: ErrorTypes) {
  let red_error = "\x1b[91merror\x1b[0m";
  match err {
      //ErrorTypes::FmtError(e) => {
      //    eprintln!("{}: {}: {}", red_error, type_err, e);
      //}
      ErrorTypes::IoError(e) => {
          eprintln!("{}: {}: {}", red_error, type_err, e);
      }
      ErrorTypes::ErrorError(e) => {
          eprintln!("{}: {}: {}", red_error, type_err, e);
      }
      ErrorTypes::StringError(e) => {
          eprintln!("{}: {}: {}", red_error, type_err, e);
      }
  }
}
pub fn throw_error(type_err: ErrorNames, err: ErrorTypes) {
  show_error(&type_err, err);
  std::process::exit(1);
}
pub fn throw_multiple_errors(type_err: ErrorNames, errs: Vec<ErrorTypes>) {
  println!("{}", errs.len());
  for err in errs {
      show_error(&type_err, err);
  };
  std::process::exit(1);
}