use std::cmp::Ordering;

use crate::{
  util::{OnError, OnSome},
  Decode, Encode, StructTag,
};

#[derive(Clone, Copy, PartialEq, Eq, Ord, Debug, Hash)]
pub struct Position {
  pub line: usize,
  pub column: usize,
}
impl PartialOrd for Position {
  fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
    if self.line < other.line {
      return Some(Ordering::Less);
    } else if self.line > other.line {
      return Some(Ordering::Greater);
    } else if self.column < other.column {
      return Some(Ordering::Less);
    } else if self.column > other.column {
      return Some(Ordering::Greater);
    } else {
      return Some(Ordering::Equal);
    }
  }
}
impl Encode for Position {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Position as u8];
    encode.extend(self.line.encode()?);
    encode.extend(self.column.encode()?);
    Ok(encode)
  }
}
impl Decode for Position {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Position as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba una posicion".to_string())?;
    return Ok(Self {
      line: usize::decode(vec)?,
      column: usize::decode(vec)?,
    });
  }
}

#[derive(Clone, PartialEq, Eq, Ord, Debug, Hash)]
pub struct Location {
  pub start: Position,
  pub end: Position,
  pub length: usize,
  pub file_name: String,
}

impl PartialOrd for Location {
  fn partial_cmp(&self, other: &Location) -> Option<Ordering> {
    if self.start < other.start {
      return Some(Ordering::Less);
    } else if self.start > other.start {
      return Some(Ordering::Greater);
    } else if self.end < other.end {
      return Some(Ordering::Less);
    } else if self.end > other.end {
      return Some(Ordering::Greater);
    } else {
      return Some(Ordering::Equal);
    }
  }
}
impl Encode for Location {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Location as u8];
    encode.extend(self.file_name.encode()?);
    encode.extend(self.start.encode()?);
    encode.extend(self.end.encode()?);
    encode.extend(self.length.encode()?);
    Ok(encode)
  }
}
impl Decode for Location {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Location as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba una locacion".to_string())?;
    return Ok(Self {
      file_name: String::decode(vec)?,
      start: Position::decode(vec)?,
      end: Position::decode(vec)?,
      length: usize::decode(vec)?,
    });
  }
}

#[derive(Clone, Debug)]
pub struct Token<TokenKind> {
  pub token_type: TokenKind,
  pub value: String,
  pub location: Location,
}

pub type TokenOptionsCallbackFull<TK> =
  fn(ch: char, start_pos: Position, line: &str, file_name: &str) -> (Token<TK>, usize);
pub type TokenOptionsCallbackChar<TK> = fn(char: char) -> TK;
pub type TokenOptionsCallbackMin<TK> = fn() -> TK;

pub enum TokenOptionCondition {
  Chars(&'static str),
  Fn(fn(char) -> bool),
}

pub enum TokenOptionResult<TK> {
  Full(TokenOptionsCallbackFull<TK>),
  Char(TokenOptionsCallbackChar<TK>),
  Min(TokenOptionsCallbackMin<TK>),
}

pub type TokenOption<'a, TK> = (TokenOptionCondition, TokenOptionResult<TK>);

pub fn tokenize<TK>(
  input: &str,
  options: Vec<TokenOption<TK>>,
  file_name: &str,
) -> Result<Vec<Token<TK>>, Box<dyn std::error::Error>> {
  let lines = input.lines();
  let mut tokens = Vec::new();
  for (line_number, line) in lines.enumerate() {
    let mut column = 0;
    while column < line.len() {
      let c = line.chars().nth(column);
      if c.is_none() {
        break;
      }
      let c = c.unwrap();
      let mut token: Option<Token<TK>> = None;
      for (condition, result) in &options {
        let is_valid = match condition {
          TokenOptionCondition::Chars(chars) => chars.contains(c),
          TokenOptionCondition::Fn(f) => f(c),
        };
        if !is_valid {
          continue;
        }
        let position = Position {
          line: line_number,
          column,
        };
        let (t, consumed) = match result {
          TokenOptionResult::Full(f) => f(c, position, line, file_name),
          TokenOptionResult::Char(f) => {
            let token_type = f(c);
            (
              Token {
                token_type,
                value: c.to_string(),
                location: Location {
                  start: position,
                  end: Position {
                    line: line_number,
                    column: column + 1,
                  },
                  length: 1,
                  file_name: file_name.to_string(),
                },
              },
              0,
            )
          }
          TokenOptionResult::Min(f) => {
            let token_type = f();
            (
              Token {
                token_type,
                value: c.to_string(),
                location: Location {
                  start: position,
                  end: Position {
                    line: line_number,
                    column: column + 1,
                  },
                  length: 1,
                  file_name: file_name.to_string(),
                },
              },
              0,
            )
          }
        };
        token = Some(t);
        column += consumed;
        break;
      }
      if let Some(token) = token {
        tokens.push(token);
      } else {
        return Err(format!("'{}'", c).into());
      }
      column += 1;
    }
  }
  Ok(tokens)
}
