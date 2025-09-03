use crate::util::{OnError, OnSome};

#[derive(PartialEq, Eq)]
pub(crate) enum StructTag {
  EndOfBlock,
  Byte,
  Compile,
  USize,

  String,
  Number,
  Bool,
  Function,
  Char,
  Null,
  Never,
  Map,
  Array,
  Class,
  Lazy,

  ChunkGroup,
  Chunk,
  Values,
  Code,
  Lines,
  Location,
  Position,

  None,
}
impl From<u8> for StructTag {
  fn from(value: u8) -> Self {
    match value {
      x if x == StructTag::EndOfBlock as u8 => StructTag::EndOfBlock,
      x if x == StructTag::Byte as u8 => StructTag::Byte,
      x if x == StructTag::Compile as u8 => StructTag::Compile,
      x if x == StructTag::USize as u8 => StructTag::USize,

      x if x == StructTag::String as u8 => StructTag::String,
      x if x == StructTag::Number as u8 => StructTag::Number,
      x if x == StructTag::Bool as u8 => StructTag::Bool,
      x if x == StructTag::Function as u8 => StructTag::Function,
      x if x == StructTag::Char as u8 => StructTag::Char,
      x if x == StructTag::Null as u8 => StructTag::Null,
      x if x == StructTag::Never as u8 => StructTag::Never,
      x if x == StructTag::Map as u8 => StructTag::Map,
      x if x == StructTag::Array as u8 => StructTag::Array,
      x if x == StructTag::Class as u8 => StructTag::Class,

      x if x == StructTag::ChunkGroup as u8 => StructTag::ChunkGroup,
      x if x == StructTag::Chunk as u8 => StructTag::Chunk,
      x if x == StructTag::Values as u8 => StructTag::Values,
      x if x == StructTag::Code as u8 => StructTag::Code,
      x if x == StructTag::Lines as u8 => StructTag::Lines,
      x if x == StructTag::Location as u8 => StructTag::Location,
      x if x == StructTag::Position as u8 => StructTag::Position,

      _ => StructTag::None,
    }
  }
}
pub(crate) trait Encode {
  fn encode(&self) -> Result<Vec<u8>, String>;
}

impl Encode for usize {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![];

    let raw = self.to_le_bytes();
    let trimmed = raw.iter().rev().skip_while(|&&b| b == 0).count();
    let len = trimmed.max(1);

    encode.push(StructTag::USize as u8);
    encode.push(len as u8);
    encode.extend(&raw[..len]);

    Ok(encode)
  }
}
impl Encode for bool {
  fn encode(&self) -> Result<Vec<u8>, String> {
    Ok(vec![StructTag::Bool as u8, if *self { 1 } else { 0 }])
  }
}
impl Encode for String {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::String as u8];

    encode.extend(
      self
        .replace('\\', "\\\\") // para poder usar caracteres de control sin problemas
        .replace('\0', "\\0")
        .replace('\x01', "\\x01")
        .as_bytes(),
    );
    encode.push(StructTag::EndOfBlock as u8);

    Ok(encode)
  }
}
impl Encode for char {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Char as u8];
    encode.extend((*self as u32).to_le_bytes());
    Ok(encode)
  }
}
pub(crate) trait Decode
where
  Self: Encode,
{
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String>
  where
    Self: Sized;
}
impl Decode for String {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::String as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un texto".to_string())?;

    let mut bytes = vec![];
    loop {
      let byte = vec
        .pop_front()
        .on_error(|_| "Binario corrupto".to_string())?;
      if byte == StructTag::EndOfBlock as u8 {
        break;
      }
      bytes.push(byte);
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
  }
}
impl Decode for bool {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    let value = vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Bool as u8 {
          None
        } else {
          vec.pop_front()
        }
      })
      .on_error(|_| "Se esperaba un buleano".to_string())?;
    if value == 0 {
      Ok(false)
    } else {
      Ok(true)
    }
  }
}
impl Decode for usize {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    let len = vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::USize as u8 {
          None
        } else {
          vec.pop_front()
        }
      })
      .on_error(|_| "Se esperaba un usize".to_string())?;
    let mut raw = [0u8; std::mem::size_of::<usize>()];
    if len as usize >= std::mem::size_of::<usize>() {
      Err("Binario corrupto".to_string())?
    }
    for i in 0..len {
      match vec.pop_front() {
        Some(byte) => raw[i as usize] = byte,
        None => return Err("Binario corrupto".to_string()),
      }
    }
    Ok(usize::from_le_bytes(raw))
  }
}
impl Decode for char {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Char as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un char".to_string())?;
    let mut raw = [0u8; std::mem::size_of::<char>()];
    for i in 0..4 {
      match vec.pop_front() {
        Some(byte) => raw[i as usize] = byte,
        None => return Err("Binario corrupto".to_string()),
      }
    }
    let u32 = u32::from_be_bytes(raw);
    char::from_u32(u32).on_error(|_| "Se esperaba un char".to_string())
  }
}
