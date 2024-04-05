#[derive(PartialEq)]
pub enum TokenType {
    Identifier,
    Number,
    String,
    Operator,
    Punctuation,
    Keyword,
    Error,
    None,
    EOF,
}
impl std::fmt::Display for TokenType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TokenType::Identifier => write!(f, "Identifier"),
      TokenType::Number => write!(f, "Number"),
      TokenType::String => write!(f, "String"),
      TokenType::Operator => write!(f, "Operator"),
      TokenType::Punctuation => write!(f, "Punctuation"),
      TokenType::Keyword => write!(f, "Keyword"),
      TokenType::Error => write!(f, "Error"),
      TokenType::None => write!(f, "None"),
      TokenType::EOF => write!(f, "EOF"),
    }
  }
}

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const OPERATORS: &str = "+-*/%=&|<>!^~";

// is for keywords and identifiers
fn is_alpha(c: char) -> bool {
  c.is_alphabetic() || c == '_' || c == '$' || c.is_numeric()
}
// only for numbers
fn is_constant(c: char) -> bool {
  c == 'i' || c == 'e' || c == 'π'
}
fn is_number(c: char, use_dot: bool) -> bool {
  c.is_numeric() || (use_dot && c == '.') || is_constant(c)
}

fn token_number(c: char, pos: util::Position, line: String) -> (util::Token<TokenType>, usize) {
  let col = pos.column;
  let mut i = col;
  let mut use_dot = c == '.';
  while i < line.len() {
    if !is_number(line.chars().nth(i).unwrap(), use_dot) {
      break;
    }
    if line.chars().nth(i).unwrap() == '.' {
      use_dot = false;
    }
    i += 1;
  }
  let token = util::Token { token_type: TokenType::Number, position: pos, value: line[col..i].to_string() };
  (token, i-col-1)
}
fn token_identifier(c: char, pos: util::Position, line: String) -> (util::Token<TokenType>, usize) {
  let col = pos.column;
  let mut i = col;
  while i < line.len() {
    if !is_alpha(line.chars().nth(i).unwrap()) {
      break;
    }
    i += 1;
  }
  let token = util::Token { token_type: TokenType::Identifier, position: pos, value: line[col..i].to_string() };
  (token, i-col-1)
}

pub fn tokenizer(input: String, file: String) -> Vec<util::Token<TokenType>> {
  let tokens = util::tokenize::<TokenType>(input, vec![
    (" \t", |c, p, _| (util::Token { token_type: TokenType::None, position: p, value: c.to_string() }, 0)),
    ((format!("{}_$", LETTERS)).as_str(), token_identifier),
    (NUMBERS, token_number),
    (OPERATORS, |c, pos, line| (util::Token { token_type: TokenType::Operator, position: pos, value: c.to_string() }, 0)),
  ]);
  match tokens {
    Ok(mut t) => {
      t.push(util::Token { token_type: TokenType::EOF, position: util::Position { line: 0, column: 0 }, value: "".to_string() });
      t.retain(|x| x.token_type != TokenType::None);
      t
    },
    Err(e) => {
      crate::throw_error("caracter invalido", crate::ErrorTypes::ErrorError(e));
      return Vec::new();
    }
  }
}