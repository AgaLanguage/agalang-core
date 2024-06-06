mod keyword;
pub use keyword::KeywordsType;

#[derive(PartialEq, Clone, Copy)]
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