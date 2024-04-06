#[derive(PartialEq)]
pub enum TokenType {
    Identifier, // variable names, function names, etc
    NumberLiteral, // 123, 123.456, 123i, 123e, 123π, etc
    StringLiteral, // 'hello world'
    Number, // 0b1010, 0x1A, 0o12, 0$17$e, etc
    String, // "hello {variable}"
    Operator, // +, -, *, /, %, &, |, ^, ~, !, =, <, >
    Punctuation, // (, ), {, }, [, ], ,, ;, :
    Keyword,
    Error,
    None,
    EOF,
}
impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::NumberLiteral => write!(f, "NumberLiteral"),
            TokenType::StringLiteral => write!(f, "StringLiteral"),
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