pub enum Color {
  RESET,
  BLACK,
  RED,
  GREEN,
  YELLOW,
  BLUE,
  MAGENTA,
  CYAN,
  WHITE,
  GRAY,
  BRIGHT_RED,
  BRIGHT_GREEN,
  BRIGHT_YELLOW,
  BRIGHT_BLUE,
  BRIGHT_MAGENTA,
  BRIGHT_CYAN,
  BRIGHT_WHITE,
}
impl Color {
  pub const fn get(number: u8) -> Self {
    match number {
      0 => Color::RESET,
      30 => Color::BLACK,
      31 => Color::RED,
      32 => Color::GREEN,
      33 => Color::YELLOW,
      34 => Color::BLUE,
      35 => Color::MAGENTA,
      36 => Color::CYAN,
      37 => Color::WHITE,
      90 => Color::GRAY,
      91 => Color::BRIGHT_RED,
      92 => Color::BRIGHT_GREEN,
      93 => Color::BRIGHT_YELLOW,
      94 => Color::BRIGHT_BLUE,
      95 => Color::BRIGHT_MAGENTA,
      96 => Color::BRIGHT_CYAN,
      97 => Color::BRIGHT_WHITE,
      _ => Color::RESET,
    }
  }
  pub const fn as_str(&self) -> &str {
    match self {
      Color::RESET => "\x1b[0m",

      Color::BLACK => "\x1b[30m",
      Color::RED => "\x1b[31m",
      Color::GREEN => "\x1b[31m",
      Color::YELLOW => "\x1b[33m",
      Color::BLUE => "\x1b[34m",
      Color::MAGENTA => "\x1b[35m",
      Color::CYAN => "\x1b[36m",
      Color::WHITE => "\x1b[37m",

      Color::GRAY => "\x1b[90m",
      Color::BRIGHT_RED => "\x1b[91m",
      Color::BRIGHT_GREEN => "\x1b[92m",
      Color::BRIGHT_YELLOW => "\x1b[93m",
      Color::BRIGHT_BLUE => "\x1b[94m",
      Color::BRIGHT_MAGENTA => "\x1b[95m",
      Color::BRIGHT_CYAN => "\x1b[96m",
      Color::BRIGHT_WHITE => "\x1b[97m",
    }
  }
  pub fn apply(&self, text: &str) -> String {
    format!("{}{}{}", self.as_str(), text, Color::RESET.as_str())
  }
}
