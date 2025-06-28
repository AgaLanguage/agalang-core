#![allow(dead_code)]
pub enum Color {
  Reset,
  Black,
  DarkRed,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,
  Gray,
  Red,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  Pink,
  BrightCyan,
  BrightWhite,
  Bold,
  Dim,
  Italic,
  Underline,
}
impl Color {
  pub const fn get(number: u8) -> Self {
    match number {
      01 => Color::Bold,
      02 => Color::Dim,
      03 => Color::Italic,
      04 => Color::Underline,
      30 => Color::Black,
      31 => Color::DarkRed,
      32 => Color::Green,
      33 => Color::Yellow,
      34 => Color::Blue,
      35 => Color::Magenta,
      36 => Color::Cyan,
      37 => Color::White,
      90 => Color::Gray,
      91 => Color::Red,
      92 => Color::BrightGreen,
      93 => Color::BrightYellow,
      94 => Color::BrightBlue,
      95 => Color::Pink,
      96 => Color::BrightCyan,
      97 => Color::BrightWhite,
      _ => Color::Reset,
    }
  }
  pub const fn as_str(&self) -> &str {
    match self {
      Color::Reset => "\x1b[0m",
      Color::Bold => "\x1b[1m",
      Color::Dim => "\x1b[2m",
      Color::Italic => "\x1b[3m",
      Color::Underline => "\x1b[4m",

      Color::Black => "\x1b[30m",
      Color::DarkRed => "\x1b[31m",
      Color::Green => "\x1b[32m",
      Color::Yellow => "\x1b[33m",
      Color::Blue => "\x1b[34m",
      Color::Magenta => "\x1b[35m",
      Color::Cyan => "\x1b[36m",
      Color::White => "\x1b[37m",

      Color::Gray => "\x1b[90m",
      Color::Red => "\x1b[91m",
      Color::BrightGreen => "\x1b[92m",
      Color::BrightYellow => "\x1b[93m",
      Color::BrightBlue => "\x1b[94m",
      Color::Pink => "\x1b[95m",
      Color::BrightCyan => "\x1b[96m",
      Color::BrightWhite => "\x1b[97m",
    }
  }
  pub fn apply(&self, text: &str) -> String {
    format!("{}{}{}", self.as_str(), text, Color::Reset.as_str())
  }
}
pub trait SetColor {
  fn set_color(&self, color: Color) -> String;
}
impl<T> SetColor for T
where
  T: ToString,
{
  fn set_color(&self, color: Color) -> String {
    color.apply(&self.to_string())
  }
}
