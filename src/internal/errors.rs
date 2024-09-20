pub enum ErrorTypes {
    //FmtError(std::fmt::Error),
    IoError(std::io::Error),
    ErrorError(Box<dyn std::error::Error>),
    StringError(String),
}
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorNames {
    None,
    PathError,
    LexerError,
    SyntaxError,
    CustomError(&'static str),
    EnviromentError,
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
            ErrorNames::EnviromentError => write!(f, "Error de entorno"),
            ErrorNames::CustomError(s) => write!(f, "{s}"),
        }
    }
}

const RED_ERROR: &str = "\x1b[1m\x1b[91merror\x1b[39m:\x1b[0m";

pub fn error_to_string(type_err: &ErrorNames, err: ErrorTypes) -> String {
    let err = match err {
        //ErrorTypes::FmtError(e) => format!("{}: {}", type_err, e),
        ErrorTypes::IoError(e) => format!("{}", e),
        ErrorTypes::ErrorError(e) => format!("{}", e),
        ErrorTypes::StringError(e) => format!("{}", e),
    };
    match type_err {
        ErrorNames::None => err,
        _ => format!("{type_err}: {err}"),
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
