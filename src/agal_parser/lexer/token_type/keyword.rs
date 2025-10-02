#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeywordsType {
  None, // invalid keyword
  Define,
  Constant,
  Function,
  Class,
  Extend,
  Public,
  Static,

  For,
  If,
  Else,
  Do,
  While,
  Try,
  Catch,
  Finally,

  Export,
  Import,
  As,
  Async,
  Name,
  Throw,
  Break,
  Return,
  Continue,
  Lazy,
  Await,
  Console,
  Delete,
}
type KeywordsList = [KeywordsType; 29];
const KEYWORDS: KeywordsList = [
  KeywordsType::None,
  KeywordsType::Delete,
  KeywordsType::Define,
  KeywordsType::Constant,
  KeywordsType::Name,
  KeywordsType::Function,
  KeywordsType::If,
  KeywordsType::Else,
  KeywordsType::Do,
  KeywordsType::While,
  KeywordsType::For,
  KeywordsType::Break,
  KeywordsType::Return,
  KeywordsType::Continue,
  KeywordsType::Class,
  KeywordsType::Static,
  KeywordsType::Public,
  KeywordsType::Extend,
  KeywordsType::Try,
  KeywordsType::Catch,
  KeywordsType::Finally,
  KeywordsType::Export,
  KeywordsType::Import,
  KeywordsType::As,
  KeywordsType::Throw,
  KeywordsType::Lazy,
  KeywordsType::Await,
  KeywordsType::Async,
  KeywordsType::Console,
];
impl KeywordsType {
  pub const fn iter() -> KeywordsList {
    KEYWORDS
  }
  pub const fn as_str(&self) -> &str {
    match self {
      KeywordsType::None => "none",
      KeywordsType::Define => "def",
      KeywordsType::Constant => "const",
      KeywordsType::Name => "nombre",
      KeywordsType::Function => "fn",
      KeywordsType::If => "si",
      KeywordsType::Else => "ent",
      KeywordsType::Do => "haz",
      KeywordsType::While => "mien",
      KeywordsType::For => "para",
      KeywordsType::Break => "rom",
      KeywordsType::Return => "ret",
      KeywordsType::Continue => "cont",
      KeywordsType::Class => "clase",
      KeywordsType::Static => "est",
      KeywordsType::Public => "pub",
      KeywordsType::Extend => "extiende",
      KeywordsType::Try => "intenta",
      KeywordsType::Catch => "captura",
      KeywordsType::Finally => "finalmente",
      KeywordsType::Export => "exporta",
      KeywordsType::Import => "importa",
      KeywordsType::As => "como",
      KeywordsType::Throw => "lanza",
      KeywordsType::Lazy => "vago",
      KeywordsType::Await => "espera",
      KeywordsType::Async => "asinc",
      KeywordsType::Console => "csl",
      KeywordsType::Delete => "borra",
    }
  }
}
impl std::fmt::Display for KeywordsType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
