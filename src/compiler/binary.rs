use crate::{
  compiler::{chunk::Chunk, ChunkGroup, Compiler, Function, Object, Value},
  util::{Location, Position},
};

trait BinaryOps {
  fn encode(self) -> Vec<u8>;
}

impl BinaryOps for &str {
  fn encode(self) -> Vec<u8> {
    let mut vec = vec![StructTag::SOB as u8];
    vec.extend_from_slice(
      self
        .replace('\\', "\\\\") // para poder usar caracteres de control sin problemas
        .replace('\0', "\\0")
        .replace('\x01', "\\x01")
        .as_bytes(),
    );
    vec.push(StructTag::EOB as u8);
    vec
  }
}

enum StructTag {
  SOB,
  EOB,
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

  ChunkGroup,
  Chunk,
  Values,
  Code,
  Lines,
  Location,
  Position,
}
type BinResult = Result<Vec<u8>, String>;
pub struct Encode;
impl Encode {
  pub fn compile(compile: &Compiler) -> BinResult {
    let mut encode = vec![];

    encode.push(StructTag::Compile as u8);
    encode.extend(Self::string(&compile.path)?);
    encode.extend(Self::function(&compile.function)?);

    Ok(encode)
  }
  pub fn string(string: &str) -> BinResult {
    let mut encode = vec![];

    encode.push(StructTag::String as u8);
    encode.extend(string.encode());

    Ok(encode)
  }
  pub fn number(value: &str) -> BinResult {
    let mut encode = vec![];

    encode.push(StructTag::Number as u8);
    encode.extend(value.encode());

    Ok(encode)
  }
  pub fn usize(usize: &usize) -> BinResult {
    let mut encode = vec![];

    let raw = usize.to_le_bytes();
    let trimmed = raw.iter().rev().skip_while(|&&b| b == 0).count();
    let len = trimmed.max(1);

    encode.push(StructTag::USize as u8);
    encode.push(len as u8);
    encode.extend(&raw[..len]);

    Ok(encode)
  }
  pub fn bool(value: bool) -> BinResult {
    let mut encode = vec![];

    encode.push(StructTag::Bool as u8);
    encode.push(if value { 1 } else { 0 });

    Ok(encode)
  }
  pub fn function(function: &Function) -> BinResult {
    let mut encode = vec![StructTag::Function as u8];
    match function {
      Function::Function {
        arity,
        chunk,
        name,
        is_async,
        location,
        has_rest,
        ..
      } => {
        encode.push(0);
        encode.extend(Self::string(name)?);
        encode.extend(Self::chunk_group(chunk)?);
        encode.extend(Self::usize(arity)?);
        encode.extend(Self::bool(*is_async)?);
        encode.extend(Self::location(location)?);
        encode.extend(Self::bool(*has_rest)?);
      }
      Function::Script { chunk, path, .. } => {
        encode.push(1);
        encode.extend(Self::string(path)?);
        encode.extend(Self::chunk_group(chunk)?);
      }
      Function::Native { .. } => return Err("No se puede compilar una funcion nativa".to_string()),
    };

    Ok(encode)
  }
  pub fn chunk_group(chunk_group: &ChunkGroup) -> BinResult {
    let mut encode = vec![StructTag::ChunkGroup as u8, StructTag::SOB as u8];

    for chunk in chunk_group.get_chunks() {
      encode.extend(Self::chunk(chunk)?)
    }

    encode.push(StructTag::EOB as u8);
    Ok(encode)
  }
  pub fn chunk(chunk: &Chunk) -> BinResult {
    let mut encode = vec![StructTag::Chunk as u8, StructTag::SOB as u8];
    {
      encode.push(StructTag::Values as u8);
      encode.push(StructTag::SOB as u8);
      for (_, value) in chunk.constants.enumerate() {
        encode.extend(Self::value(value)?);
      }
      encode.push(StructTag::EOB as u8);
    };
    {
      encode.push(StructTag::Code as u8);
      encode.push(StructTag::SOB as u8);
      for byte in &chunk.code {
        let use_byte = match *byte {
          x if x == StructTag::SOB as u8 => true,
          x if x == StructTag::EOB as u8 => true,
          x if x == StructTag::Byte as u8 => true,
          _ => false,
        };
        if use_byte {
          encode.push(StructTag::Byte as u8);
        }
        encode.push(*byte);
      }
      encode.push(StructTag::EOB as u8);
    };
    {
      encode.push(StructTag::Lines as u8);
      encode.push(StructTag::SOB as u8);
      for line in &chunk.lines {
        encode.extend(Self::usize(line)?);
      }
      encode.push(StructTag::EOB as u8);
    };

    encode.push(StructTag::EOB as u8);
    Ok(encode)
  }
  pub fn value(value: &Value) -> BinResult {
    match value {
      Value::Byte(byte) => Ok(vec![StructTag::Byte as u8, *byte]),
      Value::False => Self::bool(false),
      Value::True => Self::bool(true),
      Value::Number(number) => Self::number(&number.to_string()),
      Value::String(string) => Self::string(string),
      Value::Object(object) => Self::object(object),
      Value::Iterator(_) => Err("No se pueden compilar iteradores".to_string()),
      Value::Promise(_) => Err("No se pueden compilar promesas".to_string()),
      Value::Ref(_) => Err("No se pueden compilar referencias".to_string()),
      Value::Char(c) => {
        let mut encode = vec![StructTag::Char as u8];
        encode.extend((*c as u32).to_le_bytes());
        Ok(encode)
      }
      Value::Null => Ok(vec![StructTag::Null as u8]),
      Value::Never => Ok(vec![StructTag::Never as u8]),
    }
  }
  pub fn object(object: &Object) -> BinResult {
    match object {
      Object::Map(_, _) => Ok(vec![StructTag::Map as u8]),
      Object::Function(function) => Self::function(&*function.borrow()),
      Object::Array(_) => Ok(vec![StructTag::Array as u8]),
      Object::Class(c) => {
        let mut encode = vec![StructTag::Class as u8];
        encode.extend(Self::string(c.borrow().get_type())?);
        Ok(encode)
      }
    }
  }
  pub fn location(location: &Location) -> BinResult {
    let mut encode = vec![StructTag::Location as u8];
    encode.extend(Self::string(&location.file_name)?);
    encode.extend(Self::position(&location.start)?);
    encode.extend(Self::position(&location.end)?);
    encode.extend(Self::usize(&location.length)?);
    Ok(encode)
  }
  pub fn position(position: &Position) -> BinResult {
    let mut encode = vec![StructTag::Position as u8];
    encode.extend(Self::usize(&position.line)?);
    encode.extend(Self::usize(&position.column)?);
    Ok(encode)
  }
}
