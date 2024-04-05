mod frontend;

enum ErrorTypes {
    FmtError(std::fmt::Error),
    IoError(std::io::Error),
    ErrorError(Box<dyn std::error::Error>),
}

fn throw_error(type_err: &str, err: ErrorTypes) {
    let red_error = "\x1b[91merror\x1b[0m";
    match err {
        ErrorTypes::FmtError(e) => {
            eprintln!("{}: {}: {}", red_error, type_err, e);
        }
        ErrorTypes::IoError(e) => {
            eprintln!("{}: {}: {}", red_error, type_err, e);
        }
        ErrorTypes::ErrorError(e) => {
            eprintln!("{}: {}: {}", red_error, type_err, e);
        }
    }
    std::process::exit(1);
}

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
            throw_error("ruta inexistente", ErrorTypes::IoError(err));
            return;
        }
    };
    let tokens = frontend::tokenizer(contents, filename.to_string());
    for token in tokens {
        println!("{}: {}", token.token_type, token.value);
    }
}
