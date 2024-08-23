#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OperatorType {
    // + - * / % & | ^ ~ ! = < >
    None,
    Plus,
    Minus,
    Star,
    Division,
    Module,
    And,
    Or,
    Power,
    Negative,
    Not,
    Equals,
    LessThan,
    GreaterThan,
    QuestionMark,
}
impl OperatorType {
    pub fn from(c: char) -> Self {
        match c {
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Star,
            '/' => Self::Division,
            '%' => Self::Module,
            '&' => Self::And,
            '|' => Self::Or,
            '^' => Self::Power,
            '~' => Self::Negative,
            '!' => Self::Not,
            '=' => Self::Equals,
            '<' => Self::LessThan,
            '>' => Self::GreaterThan,
            '?' => Self::QuestionMark,
            _ => Self::None,
        }
    }
}
