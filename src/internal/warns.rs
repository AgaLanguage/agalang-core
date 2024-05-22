use super::errors::{ErrorNames, ErrorTypes, error_to_string};

const YELLOW_WARNING: &str = "\x1b[1m\x1b[93madvertencia\x1b[39m:\x1b[0m";

pub fn show_warn(type_err: &ErrorNames, err: ErrorTypes) {
    let data = error_to_string(type_err, err);
    print_warn(data);
}
pub fn show_multiple_warns(type_err: ErrorNames, errs: Vec<ErrorTypes>) {
    for err in errs {
        show_warn(&type_err, err);
    }
}
pub fn print_warn(data: String) {
    println!("{YELLOW_WARNING} {}", data);
}