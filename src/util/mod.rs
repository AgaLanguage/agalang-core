mod colors;
pub mod list;
mod tokenize;

pub use colors::*;
pub use list::*;
pub use tokenize::*;
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
    if start.line > line_number {
      continue;
    }
    if end.line < line_number {
      break;
    }
    for (char_number, char) in line.char_indices() {
      if start.column > char_number {
        continue;
      }
      if end.column < char_number {
        break;
      }
      string.push(char);
    }
  }
  Some(string)
}
pub trait OnError<T, E, V> {
  fn on_error(self, error: impl FnOnce(V) -> E) -> Result<T, E>;
}
impl<T, E> OnError<T, E, Option<T>> for Option<T> {
  fn on_error(self, error: impl FnOnce(Option<T>) -> E) -> Result<T, E> {
    match self {
      Some(v) => Ok(v),
      None => Err(error(None)),
    }
  }
}
impl<T, E, V> OnError<T, E, V> for Result<T, V> {
  fn on_error(self, error: impl FnOnce(V) -> E) -> Result<T, E> {
    match self {
      Ok(v) => Ok(v),
      Err(e) => Err(error(e)),
    }
  }
}

pub trait OnSome<T, V> {
  fn _on_some(self, some: impl FnOnce(T) -> V) -> Option<V>;
  fn _on_some_option(self, ok: impl FnOnce(T) -> Option<V>) -> Option<V>;
}

impl<T, V> OnSome<T, V> for Option<T> {
  fn _on_some(self, some: impl FnOnce(T) -> V) -> Option<V> {
    match self {
      Some(t) => Some(some(t)),
      None => None,
    }
  }
  fn _on_some_option(self, ok: impl FnOnce(T) -> Option<V>) -> Option<V> {
    match self {
      Some(t) => ok(t),
      None => None,
    }
  }
}