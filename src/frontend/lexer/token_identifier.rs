use super::{TokenType, KeywordsType};

fn get_keyword(s: &str) -> KeywordsType {
    for keyword in KeywordsType::iter() {
        if keyword.as_str() == s {
            return keyword;
        }
    }
    KeywordsType::None
}

fn is_alpha(c: char) -> bool {
  c.is_alphabetic() || c == '_' || c == '$' || c.is_numeric()
}
fn get_type_token(s: &str) -> TokenType {
    let keyword = get_keyword(s);
    if keyword == KeywordsType::None {
        return TokenType::Identifier;
    }
    TokenType::Keyword(keyword)
}

pub fn token_identifier(
    _: char,
    pos: util::Position,
    line: String,
    meta: String,
) -> (util::Token<TokenType>, usize) {
    let col = pos.column;
    let mut i = col;
    while i < line.len() {
        if !is_alpha(line.chars().nth(i).unwrap()) {
            break;
        }
        i += 1;
    }
    let s = &line[col..i];
    let token = util::Token {
        token_type: get_type_token(s),
        position: pos,
        value: s.to_string(),
        meta,
    };
    (token, i - col - 1)
}
