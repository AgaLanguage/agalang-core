pub struct Position {
    pub line: usize,
    pub column: usize,
}
impl Clone for Position {
    fn clone(&self) -> Self {
        Position {
            line: self.line,
            column: self.column,
        }
    }
}
impl Copy for Position {}

pub struct Token<TokenKind> {
    pub token_type: TokenKind,
    pub value: String,
    pub position: Position,
    pub meta: String,
}

pub type TokenOptionsCallbackFull<TK> =
    fn(ch: char, pos: Position, line: String, meta: String) -> (Token<TK>, usize);
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
    meta: String,
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
                    TokenOptionResult::Full(f) => f(c, position, line.to_string(), meta.clone()),
                    TokenOptionResult::Meta(f) => {
                        let (token_type, meta) = f(meta.clone());
                        (
                            Token {
                                token_type,
                                value: c.to_string(),
                                position,
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
                                position,
                                meta: meta.clone(),
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
                                position,
                                meta: meta.clone(),
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
