#![allow(dead_code)]
pub mod list;
mod tokenize;
mod colors;
pub mod cache;

pub use tokenize::*;
pub use colors::Color;
pub use list::*;
pub fn split_meta(meta: &str) -> (&str, &str) {
  let mut meta = meta.split("\0");
  let line = meta.next().unwrap();
  let token = meta.next();
  if token == None {
    return (line, "");
  }
  let token = token.unwrap();
  (line, token)
}
pub fn is_valid_char(valid_chars: &str, eval_char: char) -> bool {
  for c in valid_chars.chars() {
    if c == eval_char {
      return true;
    }
  }
  false
}
pub fn get_content(text: &str, start: Position, end: Position) -> Option<String> {
  if start >= end {
    return None;
  }
  let mut string = String::new();
  let lines = text.lines();
  for (line_number, line) in lines.enumerate() {
    if start.line < line_number {
      continue;
    }
    if end.line > line_number {
      break;
    }
    for (char_number, char) in line.char_indices() {
      if start.column < char_number {
        continue;
      }
      if end.line == line_number && end.column > char_number {
        break;
      }
      string.push(char);
    }
  }
  Some(string)
}
pub trait OnError<T, E, V, F: FnOnce(V) -> E> {
  fn on_error(self, error: F) -> Result<T, E>;
}
impl<T, E, F: FnOnce(Option<T>) -> E> OnError<T, E, Option<T>, F> for Option<T> {
  fn on_error(self, error: F) -> Result<T, E> {
    match self {
      Some(v) => Ok(v),
      None => Err(error(None)),
    }
  }
}
impl<T, E, V, F: FnOnce(V) -> E> OnError<T, E, V, F> for Result<T, V> {
  fn on_error(self, error: F) -> Result<T, E> {
    match self {
      Ok(v) => Ok(v),
      Err(e) => Err(error(e)),
    }
  }
}
