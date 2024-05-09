mod token_type;
pub use token_type::{KeywordsType, TokenType};
mod token_number;
use token_number::token_number;
mod token_string;
use token_string::token_string;
mod token_identifier;
use crate::{
    internal::errors::{show_error, show_multiple_errors, ErrorNames, ErrorTypes},
    util::{split_meta, to_cyan},
};
use token_identifier::token_identifier;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const OPERATORS: &str = "+-*/%=&|<>!^~?";

fn token_error(token: &util::Token<TokenType>) -> ErrorTypes {
    let (file_name, data_line, token_value) = split_meta(&token.meta);
    let line = token.position.line + 1;
    let column_token = token.position.column + 1;
    let column = column_token + token_value.len();
    let str_line = line.to_string();
    let str_init = " ".repeat(str_line.len());

    let cyan_line = to_cyan("|");
    let cyan_arrow = to_cyan("-->");

    let indicator = if token_value.len() > 0 {
        format!("{}^", "-".repeat(token_value.len()))
    } else {
        "^".to_string()
    };
    let lines = [
        format!("{}", token.value),
        format!("{}{cyan_arrow} {}:{}:{}", str_init, file_name, line, column),
        format!("{} {cyan_line}", str_init),
        format!("{} {cyan_line} {}", to_cyan(&str_line), data_line),
        format!(
            "{} {cyan_line} {}{}",
            str_init,
            " ".repeat(column_token - 1),
            to_cyan(&indicator)
        ),
        format!("{} {cyan_line}", str_init),
    ];
    let joined = lines.join("\n");
    ErrorTypes::StringError(joined)
}

pub fn tokenizer(input: String, file_name: String) -> Vec<util::Token<TokenType>> {
    let tokens = util::tokenize::<TokenType>(
        input,
        vec![
            (" \t", |c, p, _, m| {
                (
                    util::Token {
                        token_type: TokenType::None,
                        position: p,
                        value: c.to_string(),
                        meta: m,
                    },
                    0,
                )
            }),
            ((format!("{}_$", LETTERS)).as_str(), token_identifier),
            (NUMBERS, token_number),
            (OPERATORS, |c, pos, _, meta| {
                (
                    util::Token {
                        token_type: TokenType::Operator,
                        position: pos,
                        value: c.to_string(),
                        meta,
                    },
                    0,
                )
            }),
            ("'\"", token_string),
            ("(){}[],.;:", |c, pos, _, meta| {
                (
                    util::Token {
                        token_type: TokenType::Punctuation,
                        position: pos,
                        value: c.to_string(),
                        meta,
                    },
                    0,
                )
            }),
        ],
        file_name,
    );
    let tokens = match tokens {
        Ok(mut t) => {
            let end_token = t.get(t.len() -1).unwrap();
            t.push(util::Token {
                token_type: TokenType::EOF,
                position: util::Position { line: end_token.position.line, column: end_token.position.column + 1},
                value: "".to_string(),
                meta: "".to_string(),
            });
            t.retain(|x| x.token_type != TokenType::None);
            t
        }
        Err(e) => {
            show_error(&ErrorNames::LexerError, ErrorTypes::ErrorError(e));
            return Vec::new();
        }
    };
    let errors = tokens
        .iter()
        .filter(|x| x.token_type == TokenType::Error)
        .map(|x| token_error(x))
        .collect::<Vec<ErrorTypes>>();
    if errors.len() > 0 {
        show_multiple_errors(ErrorNames::LexerError, errors);
        return Vec::new();
    }
    tokens
}
