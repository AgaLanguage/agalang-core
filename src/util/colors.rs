#![allow(dead_code)]
pub enum Color {
  Reset,
  Black,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,
  Gray,
  BrightRed,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  BrightMagenta,
  BrightCyan,
  BrightWhite,
}
impl Color {
  pub const fn get(number: u8) -> Self {
    match number {
      30 => Color::Black,
      31 => Color::Red,
      32 => Color::Green,
      33 => Color::Yellow,
      34 => Color::Blue,
      35 => Color::Magenta,
      36 => Color::Cyan,
      37 => Color::White,
      90 => Color::Gray,
      91 => Color::BrightRed,
      92 => Color::BrightGreen,
      93 => Color::BrightYellow,
      94 => Color::BrightBlue,
      95 => Color::BrightMagenta,
      96 => Color::BrightCyan,
      97 => Color::BrightWhite,
      _ => Color::Reset,
    }
  }
  pub const fn as_str(&self) -> &str {
    match self {
      Color::Reset => "\x1b[0m",

      Color::Black => "\x1b[30m",
      Color::Red => "\x1b[31m",
      Color::Green => "\x1b[31m",
      Color::Yellow => "\x1b[33m",
      Color::Blue => "\x1b[34m",
      Color::Magenta => "\x1b[35m",
      Color::Cyan => "\x1b[36m",
      Color::White => "\x1b[37m",

      Color::Gray => "\x1b[90m",
      Color::BrightRed => "\x1b[91m",
      Color::BrightGreen => "\x1b[92m",
      Color::BrightYellow => "\x1b[93m",
      Color::BrightBlue => "\x1b[94m",
      Color::BrightMagenta => "\x1b[95m",
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
impl SetColor for String {
  fn set_color(&self, color: Color) -> String {
    color.apply(self)
  }
}
