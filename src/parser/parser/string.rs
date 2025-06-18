use crate::{parser, util};

fn is_alpha(c: char) -> bool {
  c.is_alphabetic() || c == '_' || c == '$' || c.is_numeric()
}
pub fn complex_string(
  token_string: util::Token<parser::TokenType>,
) -> Result<super::NodeString, super::NodeError> {
  let string = token_string.value;
  let mut result = util::List::new();
  let mut current = String::new();
  let mut is_id = false;
  let mut i = 0;
  while i < string.len() {
    let c = string.chars().nth(i);
    if c.is_none() {
      break;
    }
    let c = c.unwrap();
    i += 1;
    if c == '}' && is_id == false {
      let nc = string.chars().nth(i);
      i += 1;
      if nc == None {
        return Err(super::NodeError {
          message: "No se encontro la apertura de el identificador".to_string(),
          location: token_string.location,
        });
      }
      let nc = nc.unwrap();
      if nc == '}' {
        current.push('}');
        continue;
      }
    }
    if c != '{' && is_id == false {
      current.push(c);
      continue;
    }
    if is_id {
      if c == '}' {
        result.push(super::StringData::Id(current.clone()));
        current.clear();
        is_id = false;
        continue;
      }
      if is_alpha(c) {
        current.push(c);
        continue;
      }
    }
    let nc = string.chars().nth(i);
    i += 1;
    if nc == None {
      return Err(super::NodeError {
        message: "Se esperaba un caracter literal".to_string(),
        location: token_string.location,
      });
    }
    let nc = nc.unwrap();
    if nc == '{' {
      current.push('{');
      continue;
    }
    if nc == '}' {}
    is_id = true;
    result.push(super::StringData::Str(current.clone()));
    current.clear();
    current.push(nc);
  }
  if is_id {
    return Err(super::NodeError {
      message: "Se esperaba cierre del identificador".to_string(),
      location: token_string.location,
    });
  }
  if current.len() > 0 {
    result.push(super::StringData::Str(current));
  }
  if result.is_empty() {
    result.push(super::StringData::Str("".to_string()));
  }
  Ok(super::NodeString {
    value: result,
    location: token_string.location,
  })
}
