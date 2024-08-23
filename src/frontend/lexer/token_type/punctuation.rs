#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PunctuationType {// ( ) { } [ ] , ; : .
  None,
  CircularBracketOpen,
  CircularBracketClose,
  RegularBracketOpen,
  RegularBracketClose,
  QuadrateBracketOpen,
  QuadrateBracketClose,
  Comma,
  SemiColon,
  DoubleDot,
  Dot
}
impl PunctuationType{
  pub fn from(c: char) -> Self {
    match c {
        '(' => Self::CircularBracketOpen,
        ')' => Self::CircularBracketClose,
        '{' => Self::RegularBracketOpen,
        '}' => Self::RegularBracketClose,
        '[' => Self::QuadrateBracketOpen,
        ']' => Self::QuadrateBracketClose,
        ',' => Self::Comma,
        ';' => Self::SemiColon,
        ':' => Self::DoubleDot,
        '.' => Self::Dot,
        _ => Self::None,
    }
}
}