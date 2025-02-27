use super::{
  errors::{error_to_string, ErrorNames},
  ErrorTypes,
};

const YELLOW_WARNING: &str = "\x1b[1m\x1b[93madvertencia\x1b[39m:\x1b[0m";

pub fn show_warn(type_err: &ErrorNames, error: ErrorTypes) {
  let data = error_to_string(type_err, error);
  print_warn(data);
}
pub fn show_multiple_warns(type_err: ErrorNames, errors: Vec<ErrorTypes>) {
  for err in errors {
    show_warn(&type_err, err);
  }
}
pub fn print_warn(data: String) {
  println!("{YELLOW_WARNING} {}", data);
}
