use super::errors::{ErrorNames, ErrorTypes};

const RED_WARNING: &str = "\x1b[1m\x1b[93madvertencia\x1b[39m:\x1b[0m";

pub fn show_warn(type_err: &ErrorNames, err: ErrorTypes) {
  match err {
      //ErrorTypes::FmtError(e) => {
      //    eprintln!("{RED_WARNING} {}: {}", type_err, e);
      //}
      ErrorTypes::IoError(e) => {
          eprintln!("{RED_WARNING} {}: {}", type_err, e);
      }
      ErrorTypes::ErrorError(e) => {
          eprintln!("{RED_WARNING} {}: {}", type_err, e);
      }
      ErrorTypes::StringError(e) => {
          eprintln!("{RED_WARNING} {}: {}", type_err, e);
      }
  }
}
pub fn show_multiple_warns(type_err: ErrorNames, errs: Vec<ErrorTypes>) {
  println!("{}", errs.len());
  for err in errs {
    show_warn(&type_err, err);
  };
}