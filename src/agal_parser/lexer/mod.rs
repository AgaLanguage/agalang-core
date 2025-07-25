mod token_type;
pub use token_type::*;
mod token_number;
use token_number::token_number;
mod token_string;
use token_string::token_string;
mod token_identifier;
use crate::{
  agal_parser::show_multiple_errors,
  util::{self, Token},
};

use super::internal::ErrorTypes;
use token_identifier::token_identifier;

const NUMBERS: &str = "0123456789";
const OPERATORS: &str = "+-*/%=&|<>!^~?@";
const PUNCTUATION: &str = "(){}[],.;:";
const COLOR: util::Color = util::Color::Cyan;

fn token_error(token: &util::Token<TokenType>, source: &str) -> ErrorTypes {
  let binding = util::get_content(
    source,
    util::Position {
      line: token.location.start.line,
      column: 0,
    },
    token.location.end,
  )
  .unwrap();
  let lines = binding.lines().collect::<Vec<&str>>();
  let data_line = *lines.first().unwrap_or(&"");
  let token_value = data_line
    .chars()
    .skip(token.location.start.column)
    .collect::<String>();

  let line = token.location.start.line + 1;
  let column = token.location.end.column;
  let str_line = line.to_string();
  let str_init = " ".repeat(str_line.len());

  let cyan_line = COLOR.apply("|");
  let cyan_arrow = COLOR.apply("-->");

  let indicator = if !token_value.is_empty() {
    format!("{}^", "-".repeat(token.location.length))
  } else {
    "^".to_string()
  };
  let lines = [
    token.value.to_string(),
    format!(
      "{}{cyan_arrow} {}:{}:{}",
      str_init, token.location.file_name, line, column
    ),
    format!("{} {cyan_line}", str_init),
    format!("{} {cyan_line} {}", COLOR.apply(&str_line), data_line),
    format!(
      "{} {cyan_line} {}{}",
      str_init,
      " ".repeat(token.location.start.column),
      COLOR.apply(&indicator)
    ),
    format!("{} {cyan_line}", str_init),
  ];
  let joined = lines.join("\n");
  ErrorTypes::String(joined)
}

fn comment(
  _: char,
  position: util::Position,
  line: &str,
  file_name: &str,
) -> (util::Token<TokenType>, usize) {
  let line_len = line.len();
  let length = line_len - position.column;
  let token = util::Token {
    token_type: TokenType::None,
    value: "".to_string(),
    location: util::Location {
      start: position,
      end: util::Position {
        line: position.line,
        column: length,
      },
      length,
      file_name: file_name.to_string(),
    },
  };
  (token, length + 1)
}

pub fn tokenizer(source: &str, file_name: &str) -> (Vec<util::Token<TokenType>>, bool) {
  let tokens = util::tokenize::<TokenType>(
    source,
    vec![
      (
        util::TokenOptionCondition::Chars("\t\r "),
        util::TokenOptionResult::Min(|| TokenType::None),
      ),
      (
        util::TokenOptionCondition::Chars(NUMBERS),
        util::TokenOptionResult::Full(token_number),
      ),
      (
        util::TokenOptionCondition::Fn(|c| c.is_alphabetic() || "_$".contains(c)),
        util::TokenOptionResult::Full(token_identifier),
      ),
      (
        util::TokenOptionCondition::Chars(OPERATORS),
        util::TokenOptionResult::Char(|c| TokenType::Operator(OperatorType::from(c))),
      ),
      (
        util::TokenOptionCondition::Chars("'\""),
        util::TokenOptionResult::Full(token_string),
      ),
      (
        util::TokenOptionCondition::Chars(PUNCTUATION),
        util::TokenOptionResult::Char(|c| TokenType::Punctuation(PunctuationType::from(c))),
      ),
      (
        util::TokenOptionCondition::Chars("#"),
        util::TokenOptionResult::Full(comment),
      ),
    ],
    file_name,
  );
  let tokens = match tokens {
    Ok(mut t) => {
      let pos = t.last().map_or_else(
        || Default::default(),
        |end_token| util::Position {
          line: end_token.location.end.line,
          column: end_token.location.end.column + 1,
        },
      );
      t.push(util::Token {
        token_type: TokenType::EndOfFile,
        location: util::Location {
          start: pos,
          end: pos,
          length: 0,
          file_name: file_name.to_string(),
        },
        value: "".to_string(),
      });
      t.retain(|x| x.token_type != TokenType::None);
      t
    }
    Err((value, location)) => {
      vec![Token {
        location,
        value,
        token_type: TokenType::Error,
      }]
    }
  };
  let errors = tokens
    .iter()
    .filter(|x| x.token_type == TokenType::Error)
    .map(|x| token_error(x, source))
    .collect::<Vec<ErrorTypes>>();
  if !errors.is_empty() {
    show_multiple_errors(&crate::agal_parser::ErrorNames::LexerError, errors);
    return (vec![], true);
  }
  (tokens, false)
}
