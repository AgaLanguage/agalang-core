#![allow(warnings)]
mod runtime;
mod modules;
use std::process::ExitCode;

fn main() -> ExitCode {
    let filename = file();
    if filename.is_none() {
        return ExitCode::FAILURE;
    }
    let filename = filename.unwrap();
    let ref stack = runtime::Stack::get_default();
    let global_env = runtime::env::get_default().as_ref();

    let program = runtime::full_eval(filename, stack, global_env);
    if program.is_err() {
        return ExitCode::FAILURE;
    }
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
