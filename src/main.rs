#[warn(dead_code)]
mod frontend;
mod internal;
mod runtime;
mod util;

use std::process::ExitCode;

fn main() -> ExitCode {
    let filename = file();
    if filename.is_none() {
        return ExitCode::FAILURE;
    }
    let ref filename = filename.unwrap();
    let contents = code(filename);
    if contents.is_none() {
        return ExitCode::FAILURE;
    }
    let contents = contents.unwrap();
    let program: frontend::ast::Node = {
        let mut parser = frontend::Parser::new(contents, filename);
        parser.produce_ast()
    };
    if program.is_error() {
        let type_err = internal::errors::ErrorNames::SyntaxError;
        let node_err = program.get_error().unwrap();
        let err = frontend::node_error(&node_err);
        let data = internal::errors::error_to_string(&type_err, err);
        internal::print_error(data);
        return ExitCode::FAILURE;
    }
    println!("{}", program);
    return ExitCode::SUCCESS;
}
fn file() -> Option<String> {
    let mut args: Vec<_> = std::env::args().collect();
    args.push("file.agal".to_string());
    let args = args;
    if args.len() < 2 {
        let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
        println!("{} {} <filename>", blue_usage, args[0]);
        return None;
    }
    Some(args[1].to_string())
}
fn code(filename: &str) -> Option<String> {
    let contents = std::fs::read_to_string(filename);
    match contents {
        Ok(contents) => Some(contents),
        Err(err) => {
            let ref type_err = internal::ErrorNames::PathError;
            let err = internal::ErrorTypes::IoError(err);
            internal::show_error(type_err, err);
            None
        }
    }
}
