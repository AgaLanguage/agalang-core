mod keyword;
pub use keyword::KeywordsType;

#[derive(PartialEq)]
pub enum TokenType {
    Identifier, // variable names, function names, etc
    NumberLiteral, // 123, 123.456, 123i, 123e, 123π, etc
    StringLiteral, // 'hello world'
    Number, // 0b1010, 0x1A, 0o12, 0$17$e, etc
    String, // "hello {variable}"
    Operator, // + - * / % & | ^ ~ ! = < >
    Punctuation(char), // ( ) { } [ ] , ; : .

    Keyword(KeywordsType),
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
            TokenType::Punctuation(pun) => write!(f, "Punctuation \"{}\"", pun.to_string()),
            TokenType::Keyword(key) => write!(f, "Keyword({})", key.to_string()),
            TokenType::Error => write!(f, "Error"),
            TokenType::None => write!(f, "None"),
            TokenType::EOF => write!(f, "EOF"),
        }
    }
}
impl Clone for TokenType {
    fn clone(&self) -> TokenType {
        match self {
            TokenType::Identifier => TokenType::Identifier,
            TokenType::NumberLiteral => TokenType::NumberLiteral,
            TokenType::StringLiteral => TokenType::StringLiteral,
            TokenType::Number => TokenType::Number,
            TokenType::String => TokenType::String,
            TokenType::Operator => TokenType::Operator,
            TokenType::Punctuation(pun) => TokenType::Punctuation(*pun),
            TokenType::Keyword(key) => TokenType::Keyword(*key),
            TokenType::Error => TokenType::Error,
            TokenType::None => TokenType::None,
            TokenType::EOF => TokenType::EOF,
        }
    }
}
impl Copy for TokenType {}