use crate::util;

use super::TokenType;

fn is_hexadecimal(v: Option<&char>) -> bool {
  if v == None {
    return false;
  }
  let c = *v.unwrap();
  c.is_numeric() || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')
}
pub fn token_string(
  quote: char,
  pos: util::Position,
  line: String,
  file_name: String,
) -> (util::Token<TokenType>, usize) {
  let col = pos.column;
  let src = line.chars().collect::<Vec<char>>();
  let mut str_length = col + (1/* the quote character */);
  let mut str = String::new();
  let mut closed = false;
  while src.len() > str_length {
    let v = *src.get(str_length).unwrap();
    str_length += 1;
    if v == quote {
      closed = true;
      break;
    }
    if v == '\\' {
      let next = src.get(str_length);
      str_length += 1;
      if next == None {
        return (
          util::Token {
            token_type: TokenType::Error,
            location: util::Location {
              start: pos,
              end: util::Position {
                column: str_length,
                line: pos.line,
              },
              length: str_length - col,
              file_name: file_name.clone(),
            },
            value: "Se esperaba un caracter literal".to_string(),
            meta: format!("{line}\0{quote}{str}"),
          },
          str_length - col - 1,
        );
      }
      let next = next.unwrap().to_owned();
      if next == 'n' {
        str.push('\n');
      } else if next == 't' {
        str.push('\t');
      } else if next == 'r' {
        str.push('\r');
      } else if next == '0' {
        str.push('\0');
      } else if next == 'x' {
        let n1 = src.get(str_length);
        str_length += 1;
        let n2 = src.get(str_length);
        str_length += 1;
        if !is_hexadecimal(n1) || !is_hexadecimal(n2) {
          return (
            util::Token {
              token_type: TokenType::Error,
              location: util::Location {
                start: pos,
                end: util::Position {
                  column: str_length,
                  line: pos.line,
                },
                length: str_length - col,
                file_name: file_name.clone(),
              },
              value: "Se esperaba un numero hexadecimal".to_string(),
              meta: format!("{line}\0{quote}{str}"),
            },
            str_length - col - 1,
          );
        }
        let n1 = n1.unwrap().to_owned();
        let n2 = n2.unwrap().to_owned();
        let hex = format!("{}{}", n1, n2);
        let value = u32::from_str_radix(&hex, 16).unwrap();
        str.push(char::from_u32(value as u32).unwrap());
      } else if next == 'u' {
        let n1 = src.get(str_length);
        str_length += 1;
        let n2 = src.get(str_length);
        str_length += 1;
        let n3 = src.get(str_length);
        str_length += 1;
        let n4 = src.get(str_length);
        str_length += 1;
        if !is_hexadecimal(n1) || !is_hexadecimal(n2) || !is_hexadecimal(n3) || !is_hexadecimal(n4)
        {
          return (
            util::Token {
              token_type: TokenType::Error,
              location: util::Location {
                start: pos,
                end: util::Position {
                  column: str_length,
                  line: pos.line,
                },
                length: str_length - col,
                file_name: file_name.clone(),
              },
              value: "Se esperaba un numero hexadecimal".to_string(),
              meta: format!("{line}\0{quote}{str}"),
            },
            str_length - col - 1,
          );
        }
        let n1 = n1.unwrap().to_owned();
        let n2 = n2.unwrap().to_owned();
        let n3 = n3.unwrap().to_owned();
        let n4 = n4.unwrap().to_owned();
        let hex = format!("{}{}{}{}", n1, n2, n3, n4);
        let value = u32::from_str_radix(&hex, 16).unwrap();
        str.push(char::from_u32(value as u32).unwrap());
      } else {
        // implement '\\' and '\'' as literals
        str.push(next);
      }
    } else {
      str.push(v);
    }
  }
  if !closed {
    return (
      util::Token {
        token_type: TokenType::Error,
        location: util::Location {
          start: pos,
          end: util::Position {
            column: str_length,
            line: pos.line,
          },
          length: str_length - col,
          file_name: file_name.clone(),
        },
        value: format!("Se esperaba un [{quote}] para cerrar la cadena"),
        meta: format!("{line}\0{quote}{str}"),
      },
      str_length - col - 1,
    );
  }
  (
    util::Token {
      token_type: if quote == '\'' {
        TokenType::StringLiteral
      } else {
        TokenType::String
      },
      location: util::Location {
        start: pos,
        end: util::Position {
          column: str_length,
          line: pos.line,
        },
        length: str_length - col,
        file_name: file_name.clone(),
      },
      value: str,
      meta: file_name,
    },
    str_length - col - 1,
  )
}
