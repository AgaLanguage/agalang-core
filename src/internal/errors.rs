pub enum ErrorTypes {
    //FmtError(std::fmt::Error),
    IoError(std::io::Error),
    ErrorError(Box<dyn std::error::Error>),
    StringError(String),
}
#[derive(Clone)]
pub enum ErrorNames {
    None,
    PathError,
    LexerError,
    SyntaxError,
    CustomError(String)
}
impl std::fmt::Display for ErrorNames {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorNames::None => write!(f, ""),
            ErrorNames::PathError => write!(f, "Error ruta"),
            ErrorNames::LexerError => write!(f, "Error léxico"),
            ErrorNames::SyntaxError => write!(f, "Error sintáctico"),
            ErrorNames::CustomError(s) => write!(f, "{s}"),
        }
    }
}

const RED_ERROR: &str = "\x1b[1m\x1b[91merror\x1b[39m:\x1b[0m";

pub fn error_to_string(type_err: &ErrorNames, err: ErrorTypes) -> String {
  match err {
      //ErrorTypes::FmtError(e) => format!("{}: {}", type_err, e),
      ErrorTypes::IoError(e) => format!("{}: {}", type_err, e),
      ErrorTypes::ErrorError(e) => format!("{}: {}", type_err, e),
      ErrorTypes::StringError(e) => format!("{}: {}", type_err, e),
  }
}
pub fn show_error(type_err: &ErrorNames, err: ErrorTypes) {
    let data = error_to_string(type_err, err);
    print_error(data);
}

pub fn show_multiple_errors(type_err: &ErrorNames, errs: Vec<ErrorTypes>) {
    for err in errs {
        show_error(type_err, err);
    }
}

pub fn print_error(data: String) {
    println!("{RED_ERROR} {}", data);
}