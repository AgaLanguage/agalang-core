use crate::util;

use super::TokenType;

const fn is_constant(c: char) -> bool {
  c == 'i' || c == 'e' || c == 'π'
}
fn is_number(c: char, use_dot: bool) -> bool {
  c.is_numeric() || (!use_dot && c == '.') || is_constant(c)
}

fn number_literal(
  position: util::Position,
  line: &str,
  file_name: &str,
) -> (util::Token<TokenType>, usize) {
  let col = position.column;
  let mut i = col;
  let mut use_dot = false;
  let mut value = String::new();
  while i < line.len() {
    let c = line.chars().nth(i);
    if c.is_none() {
      break;
    }
    let c = c.unwrap();
    if c == '_' {
      i+=1;
      continue;
    }
    if !is_number(c, use_dot) {
      break;
    }
    if c == '.' {
      use_dot = true;
    }
    value.push(c);
    i += 1;
  }
  let token = util::Token {
    token_type: TokenType::NumberLiteral,
    location: util::Location {
      start: position,
      end: util::Position {
        column: i,
        line: position.line,
      },
      length: i - col,
      file_name: file_name.to_string(),
    },
    value,
  };
  (token, i - col - 1)
}

fn number_base(
  c: char,
  pos: util::Position,
  line: &str,
  file_name: &str,
) -> (util::Token<TokenType>, usize) {
  let col = pos.column;
  let mut i = col + 1;
  let mut base = 10;
  if c == '0' && i < line.len() {
    let c = line.chars().nth(i);
    if c.is_none() {
      return (
        util::Token {
          token_type: TokenType::NumberLiteral,
          location: util::Location {
            start: pos,
            end: util::Position {
              column: i,
              line: pos.line,
            },
            length: i - col,
            file_name: file_name.to_string(),
          },
          value: "0".to_string(),
        },
        i - col - 1,
      );
    }
    let c = c.unwrap();
    if c == 'b' {
      base = 2;
      i += 1;
      if let Some('y') = line.chars().nth(i) {
        i += 1;
        let mut value = String::new();
        let mut x = 0;
        while x < 8 {
          let bit = line.chars().nth(i);
          if bit.is_none() {
            break;
          }
          let bit = bit.unwrap();
          if bit == '0' || bit == '1' {
            value.push(bit);
            x += 1;
          } else if bit != '_' {
            break;
          }
          i += 1;
        }
        return if x == 0 {
          (
            util::Token {
              token_type: TokenType::Error,
              location: util::Location {
                start: pos,
                end: util::Position {
                  column: i,
                  line: pos.line,
                },
                length: i - col,
                file_name: file_name.to_string(),
              },
              value: format!("No se pudo analizar el byte"),
            },
            i - col - 1,
          )
        } else {
          (
            util::Token {
              token_type: TokenType::Byte,
              location: util::Location {
                start: pos,
                end: util::Position {
                  column: i,
                  line: pos.line,
                },
                length: i - col,
                file_name: file_name.to_string(),
              },
              value,
            },
            i - col - 1,
          )
        };
      }
    } else if c == 'o' {
      base = 8;
      i += 1;
    } else if c == 'd' {
      base = 10;
      i += 1;
    } else if c == 'x' {
      base = 16;
      i += 1;
    } else if c == 'n' {
      i += 1;
      if i >= line.len() {
        // not i < line.len()
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: pos.column,
                line: pos.line,
              },
              length: 1,
              file_name: file_name.to_string(),
            },
            value: "Se esperaba un número base".to_string(),
          },
          0,
        );
      }
      let mut base_str = String::new();
      while i < line.len() {
        let ch = match line.chars().nth(i) {
          Some(c) => c,
          None => break,
        };
        if !ch.is_digit(10) {
          break;
        }
        base_str.push(ch);
        i += 1;
      }
      if base_str.len() == 0 {
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: i,
                line: pos.line,
              },
              length: i - col,
              file_name: file_name.to_string(),
            },
            value: "Se esperaba un número base".to_string(),
          },
          i - col - 1,
        );
      }
      let base_number = base_str.parse();
      if base_number.is_err() {
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: i,
                line: pos.line,
              },
              length: i - col,
              file_name: file_name.to_string(),
            },
            value: "Se esperaba un número en base 10".to_string(),
          },
          i - col - 1,
        );
      }
      let base_number = base_number.unwrap();
      if 2 > base_number || base_number > 36 {
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: i,
                line: pos.line,
              },
              length: i - col,
              file_name: file_name.to_string(),
            },
            value: "La base debe estar entre 2 y 36".to_string(),
          },
          i - col - 1,
        );
      }
      base = base_number;
      let value_char = line.chars().nth(i);
      if value_char == None {
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: i,
                line: pos.line,
              },
              length: i - col,
              file_name: file_name.to_string(),
            },
            value: "Se esperaba un \"|\" para el valor".to_string(),
          },
          i - col - 1,
        );
      }
      if value_char.unwrap() == '|' {
        i += 1;
      }
    }
  }

  // save the first index of the value
  let value_index = i;
  let mut value = String::new();
  while i < line.len() {
    let ch = match line.chars().nth(i) {
      Some(c) => c,
      None => break,
    };
    if ch == '_' {
      i += 1;
    } else if ch.is_digit(base) {
      i += 1;
      value.push(ch);
    } else {
      break;
    }
  }
  if i - value_index == 0 {
    return (
      util::Token {
        token_type: TokenType::Error,
        location: util::Location {
          start: pos,
          end: util::Position {
            column: i,
            line: pos.line,
          },
          length: i - col,
          file_name: file_name.to_string(),
        },
        value: "Se esperaba un número".to_string(),
      },
      i - col - 1,
    );
  }
  let value = match line.get(value_index..i) {
    Some(value) => value,
    None => {
      return (
        util::Token {
          token_type: TokenType::Error,
          location: util::Location {
            start: pos,
            end: util::Position {
              column: i,
              line: pos.line,
            },
            length: i - col,
            file_name: file_name.to_string(),
          },
          value: "Se esperaba un número".to_string(),
        },
        i - col - 1,
      )
    }
  };
  let token = util::Token {
    token_type: TokenType::Number,
    location: util::Location {
      start: pos,
      end: util::Position {
        column: i,
        line: pos.line,
      },
      length: i - col,
      file_name: file_name.to_string(),
    },
    value: format!("0n{}|{}", base, value),
  };
  (token, i - col - 1)
}

pub fn token_number(
  c: char,
  pos: util::Position,
  line: &str,
  file_name: &str,
) -> (util::Token<TokenType>, usize) {
  if c == '0' {
    let next = line.chars().nth(pos.column + 1);
    if next != None && util::is_valid_char("bodxn", next.unwrap()) {
      return number_base(c, pos, line, file_name);
    }
  }
  number_literal(pos, line, file_name)
}
