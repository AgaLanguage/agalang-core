mod keyword;
pub use keyword::KeywordsType;
mod operator;
pub use operator::OperatorType;
mod punctuation;
pub use punctuation::PunctuationType;

#[derive(Clone, Copy, Debug)]
pub enum TokenType {
  Identifier,                   // variable names, function names, etc
  NumberLiteral,                // 123, 123.456, 123i, 123e, 123π, etc
  StringLiteral,                // 'hello world'
  Number,                       // 0b1010, 0x1A, 0o12, 0$17$e, etc
  String,                       // "hello {variable}"
  Operator(OperatorType),       // + - * / % & | ^ ~ ! = < >
  Punctuation(PunctuationType), // ( ) { } [ ] , ; : .
  Keyword(KeywordsType),
  Error,
  None,
  Byte, // 0by00000000
  EOF,
}
impl PartialEq for TokenType {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Identifier, Self::Identifier) => true,
      (Self::NumberLiteral, Self::NumberLiteral) => true,
      (Self::StringLiteral, Self::StringLiteral) => true,
      (Self::Number, Self::Number) => true,
      (Self::String, Self::String) => true,
      (Self::Operator(a), Self::Operator(b)) => {
        a == b || (*a == OperatorType::None) || (*b == OperatorType::None)
      }
      (Self::Punctuation(a), Self::Punctuation(b)) => {
        a == b || (*a == PunctuationType::None) || (*b == PunctuationType::None)
      }
      (Self::Keyword(a), Self::Keyword(b)) => {
        a == b || (*a == KeywordsType::None) || (*b == KeywordsType::None)
      }
      (Self::Error, Self::Error) => true,
      (Self::None, Self::None) => true,
      (Self::EOF, Self::EOF) => true,
      _ => false,
    }
  }
  fn ne(&self, other: &Self) -> bool {
    !self.eq(other)
  }
}
