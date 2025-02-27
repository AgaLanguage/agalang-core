use crate::util;

use super::{KeywordsType, TokenType};

fn get_keyword(s: &str) -> KeywordsType {
  for keyword in KeywordsType::iter() {
    if keyword.as_str() == s {
      return keyword;
    }
  }
  KeywordsType::None
}

fn is_alpha(c: char) -> bool {
  c.is_alphabetic() || c == '_' || c == '$' || c.is_ascii_digit()
}

fn get_type_token(s: &str) -> TokenType {
  let keyword = get_keyword(s);
  if keyword == KeywordsType::None {
    return TokenType::Identifier;
  }
  TokenType::Keyword(keyword)
}

pub fn token_identifier(
  ch: char,
  position: util::Position,
  ref line: String,
  file_name: String,
) -> (util::Token<TokenType>, usize) {
  let col = position.column;
  let mut i = col;
  let mut value = String::new();
  while i < line.len() {
    let c = line.chars().nth(i);
    if c.is_none() {
      break;
    }
    let c = c.unwrap();
    if !is_alpha(c) {
      break;
    }
    value.push(c);
    i += 1;
  }
  if i == col {
    return (
      util::Token {
        token_type: TokenType::None,
        location: util::Location {
          start: position,
          end: util::Position {
            column: i,
            line: position.line,
          },
          length: 1,
          file_name: file_name.clone(),
        },
        value: ch.to_string(),
        meta: file_name,
      },
      0,
    );
  }
  let token = util::Token {
    token_type: get_type_token(&value),
    location: util::Location {
      start: position,
      end: util::Position {
        column: i,
        line: position.line,
      },
      length: i - col,
      file_name: file_name.clone(),
    },
    value,
    meta: file_name,
  };

  (token, i - col - 1)
}
