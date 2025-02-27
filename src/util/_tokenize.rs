use std::cmp::Ordering;

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
#[derive(Clone, Debug)]
pub struct Token<TokenKind> {
  pub token_type: TokenKind,
  pub value: String,
  pub location: Location,
  pub meta: String,
}

pub type TokenOptionsCallbackFull<TK> =
  fn(ch: char, start_pos: Position, line: String, meta: String) -> (Token<TK>, usize);
pub type TokenOptionsCallbackMeta<TK> = fn(meta: String) -> (TK, String);
pub type TokenOptionsCallbackChar<TK> = fn(char: char) -> TK;
pub type TokenOptionsCallbackMin<TK> = fn() -> TK;

pub enum TokenOptionCondition {
  Chars(&'static str),
  Fn(fn(char) -> bool),
}

pub enum TokenOptionResult<TK> {
  Full(TokenOptionsCallbackFull<TK>),
  Meta(TokenOptionsCallbackMeta<TK>),
  Char(TokenOptionsCallbackChar<TK>),
  Min(TokenOptionsCallbackMin<TK>),
}

pub type TokenOption<'a, TK> = (TokenOptionCondition, TokenOptionResult<TK>);

pub fn tokenize<TK>(
  input: String,
  options: Vec<TokenOption<TK>>,
  file_name: String,
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
          TokenOptionResult::Full(f) => f(c, position, line.to_string(), file_name.clone()),
          TokenOptionResult::Meta(f) => {
            let (token_type, meta) = f(file_name.clone());
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
                  file_name: file_name.clone(),
                },
                meta,
              },
              0,
            )
          }
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
                  file_name: file_name.clone(),
                },
                meta: file_name.clone(),
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
                  file_name: file_name.clone(),
                },
                meta: file_name.clone(),
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
