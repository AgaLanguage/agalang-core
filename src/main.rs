mod frontend;
mod internal;
mod util;

const FAILURE: std::process::ExitCode = std::process::ExitCode::FAILURE;
const SUCCESS: std::process::ExitCode = std::process::ExitCode::SUCCESS;

fn main() -> std::process::ExitCode{
    let mut args: Vec<_> = std::env::args().collect();
    args.push("file.agal".to_string());
    let args = args;
    if args.len() < 2 {
        let blue_usage = "\x1b[94m\x1b[1mUsage\x1b[39m:\x1b[0m";
        println!("{} {} <filename>", blue_usage, args[0]);
        return FAILURE;
    }
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename);
    let contents = match contents {
        Ok(contents) => contents,
        Err(err) => {
            internal::errors::show_error(&internal::errors::ErrorNames::PathError, internal::errors::ErrorTypes::IoError(err));
            return FAILURE;
        }
    };
    let program = frontend::Parser::new(contents, filename).produce_ast();
    if program.is_error() {
        let type_err = internal::errors::ErrorNames::SyntaxError;
        let node_err = program.get_error().unwrap();
        let err = frontend::node_error(&node_err);
        internal::errors::show_error(&type_err, err);
        return FAILURE;
    }
    println!("{}", program);
    return SUCCESS;
}