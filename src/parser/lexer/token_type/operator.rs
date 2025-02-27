#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OperatorType {
  None,
  /// +
  Plus,
  /// -
  Minus,
  /// *
  Star,
  /// /
  Division,
  /// %
  Modulo,
  /// &
  And,
  /// |
  Or,
  /// ^
  Exponential,
  /// ~
  Approximate,
  /// !
  Not,
  /// =
  Equals,
  /// <
  LessThan,
  /// >
  GreaterThan,
  /// ?
  QuestionMark,
}
impl OperatorType {
  pub fn from(c: char) -> Self {
    match c {
      '+' => Self::Plus,
      '-' => Self::Minus,
      '*' => Self::Star,
      '/' => Self::Division,
      '%' => Self::Modulo,
      '&' => Self::And,
      '|' => Self::Or,
      '^' => Self::Exponential,
      '~' => Self::Approximate,
      '!' => Self::Not,
      '=' => Self::Equals,
      '<' => Self::LessThan,
      '>' => Self::GreaterThan,
      '?' => Self::QuestionMark,
      _ => Self::None,
    }
  }
}
