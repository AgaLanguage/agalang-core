use crate::frontend::ast::Node;

mod frontend;
mod internal;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        let blue_usage = "\x1b[94mUsage\x1b[0m";
        println!("{}: {} <filename>", blue_usage, args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename);
    let contents = match contents {
        Ok(contents) => contents,
        Err(err) => {
            internal::errors::throw_error(internal::errors::ErrorNames::PathError, internal::errors::ErrorTypes::IoError(err));
            return;
        }
    };
    let program = frontend::produce_ast(contents, false, filename.to_string());
    println!("{}", program.to_string());
}
