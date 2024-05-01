mod frontend;
mod internal;
mod util;
 
fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();
    args.push("file.agal".to_string());
    let args = args;
    if args.len() < 2 {
        let blue_usage = "\x1b[94mUsage\x1b[0m";
        println!("{}: {} <filename>", blue_usage, args[0]);
        std::process::exit(0);
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
    let program = frontend::Parser::new(contents, filename.to_string()).produce_ast(false);
    println!("{}", program.to_string());
}
