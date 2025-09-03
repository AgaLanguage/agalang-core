use crate::{compiler::ValueArray, Decode, Encode, MultiRefHash, StructTag};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
  // Const
  Constant,
  // Math
  Add,
  Subtract,
  Multiply,
  Divide,
  Negate,
  Modulo,
  // Expr
  Not,
  Approximate,
  At,
  AsRef,
  AsBoolean,
  AsString,
  Call,
  ArgDecl,
  GetMember,
  SetMember,
  // Binary
  And,
  Or,
  GreaterThan,
  LessThan,
  Equals,
  // Statement
  ConsoleOut,
  VarDecl,
  ConstDecl,
  DelVar,
  GetVar,
  SetVar,
  Loop,
  Import,
  Export,
  ExtendClass,
  Throw,
  Try,
  // Control
  Pop,
  Await,
  UnPromise, // obtiene el valor de una promesa
  Promised,  // mueve el frame a los asincronos
  NewLocals,
  RemoveLocals,
  JumpIfFalse,
  Jump,
  Return,
  Break,
  Continue,
  Copy,        // Para duplicar el ultimo valor en el stack (obtener el padre de un objeto)
  SetScope,    // Agrega el scope actual a el ultimo valor de la pila (para funciones)
  InClass,     // Determina que el scope actual es una clase (metodos de clase)
  GetInstance, // Para agregar las propiedades de inctancia al declarar la clase
  // Invalid
  Null,
}
impl From<&u8> for OpCode {
  fn from(value: &u8) -> Self {
    (*value).into()
  }
}
impl From<u8> for OpCode {
  fn from(value: u8) -> Self {
    match value {
      x if x == Self::Approximate as u8 => Self::Approximate,
      x if x == Self::GetMember as u8 => Self::GetMember,
      x if x == Self::SetMember as u8 => Self::SetMember,
      x if x == Self::Constant as u8 => Self::Constant,
      x if x == Self::Call as u8 => Self::Call,
      x if x == Self::Add as u8 => Self::Add,
      x if x == Self::ArgDecl as u8 => Self::ArgDecl,
      x if x == Self::Subtract as u8 => Self::Subtract,
      x if x == Self::Multiply as u8 => Self::Multiply,
      x if x == Self::Divide as u8 => Self::Divide,
      x if x == Self::Negate as u8 => Self::Negate,
      x if x == Self::Not as u8 => Self::Not,
      x if x == Self::AsBoolean as u8 => Self::AsBoolean,
      x if x == Self::AsString as u8 => Self::AsString,
      x if x == Self::GreaterThan as u8 => Self::GreaterThan,
      x if x == Self::LessThan as u8 => Self::LessThan,
      x if x == Self::Equals as u8 => Self::Equals,
      x if x == Self::ConsoleOut as u8 => Self::ConsoleOut,
      x if x == Self::GetVar as u8 => Self::GetVar,
      x if x == Self::SetVar as u8 => Self::SetVar,
      x if x == Self::VarDecl as u8 => Self::VarDecl,
      x if x == Self::ConstDecl as u8 => Self::ConstDecl,
      x if x == Self::Pop as u8 => Self::Pop,
      x if x == Self::And as u8 => Self::And,
      x if x == Self::Or as u8 => Self::Or,
      x if x == Self::Loop as u8 => Self::Loop,
      x if x == Self::NewLocals as u8 => Self::NewLocals,
      x if x == Self::RemoveLocals as u8 => Self::RemoveLocals,
      x if x == Self::JumpIfFalse as u8 => Self::JumpIfFalse,
      x if x == Self::Jump as u8 => Self::Jump,
      x if x == Self::Return as u8 => Self::Return,
      x if x == Self::Copy as u8 => Self::Copy,
      x if x == Self::SetScope as u8 => Self::SetScope,
      x if x == Self::Import as u8 => Self::Import,
      x if x == Self::Export as u8 => Self::Export,
      x if x == Self::DelVar as u8 => Self::DelVar,
      x if x == Self::Await as u8 => Self::Await,
      x if x == Self::UnPromise as u8 => Self::UnPromise,
      x if x == Self::Promised as u8 => Self::Promised,
      x if x == Self::Modulo as u8 => Self::Modulo,
      x if x == Self::InClass as u8 => Self::InClass,
      x if x == Self::ExtendClass as u8 => Self::ExtendClass,
      x if x == Self::GetInstance as u8 => Self::GetInstance,
      x if x == Self::Throw as u8 => Self::Throw,
      x if x == Self::Try as u8 => Self::Try,

      x if x == Self::At as u8 => Self::At,
      x if x == Self::AsRef as u8 => Self::AsRef,
      _ => Self::Null,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub lines: Vec<usize>,
  pub constants: super::ValueArray,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      lines: Vec::new(),
      constants: super::ValueArray::new(),
    }
  }
  pub fn read(&self, index: usize) -> u8 {
    self.code[index]
  }
  fn overwrite(&mut self, index: usize, byte: u8) {
    self.code[index] = byte;
  }
  pub fn write(&mut self, byte: u8, line: usize) {
    self.code.push(byte);
    self.lines.push(line);
  }
  pub fn write_buffer(&mut self, bytes: Vec<u8>, line: usize) {
    for byte in bytes {
      self.write(byte, line);
    }
  }
  fn add_constant(&mut self, value: super::Value) -> u8 {
    if self.constants.has_value(&value) {
      return self.constants.get_index(&value).unwrap_or(0);
    }
    self.constants.write(value);
    self.constants.len() - 1
  }
  pub fn add_loop(&mut self, loop_start: usize) -> Result<(), String> {
    self.write(OpCode::Loop as u8, self.code.len());

    let offset = self.code.len() - loop_start + 2;
    if offset > u16::MAX.into() {
      Err("Longitud muy alta".to_string())?
    }
    self.write(((offset >> 8) & 0xff) as u8, self.code.len());
    self.write((offset & 0xff) as u8, self.code.len());
    Ok(())
  }
  pub fn jump(&mut self, code: OpCode) -> usize {
    self.write_buffer(vec![code as u8, 0xFF, 0xFF], self.code.len());
    self.code.len() - 2
  }
  pub fn patch_jump(&mut self, offset: usize) -> Result<(), String> {
    let jump = self.code.len() - offset - (2/* Data bytes */);
    if jump > u16::MAX.into() {
      Err("Longitud muy alta".to_string())?
    }
    self.overwrite(offset, ((jump >> 8) & 0xff) as u8);
    self.overwrite(offset + 1, (jump & 0xff) as u8);
    Ok(())
  }
  fn _print(&self, name: String) {
    println!("===== {name} =====");

    println!("-- {name} consts -");
    println!("Index | Value",);
    for (i, value) in self.constants.enumerate() {
      println!("   {i:02x} | {value:?}");
    }
    println!("-- {name} consts -");
    println!("Byte | Operation        | JumpTo | Index | Value",);
    let mut offset = 0;
    while offset < self.code.len() {
      let i = offset;
      let op = OpCode::from(self.code[offset]);
      offset += 1;
      let (jump_to, index, value): (String, String, String) = match op {
        OpCode::Jump | OpCode::JumpIfFalse => {
          let a = self.read(offset) as u16;
          let b = self.read(offset + 1) as u16;
          offset += 2;
          (
            format!("{:04x}", (a << 8) | b),
            "--".to_string(),
            "-------------------------".to_string(),
          )
        }
        OpCode::Constant
        | OpCode::GetVar
        | OpCode::ConstDecl
        | OpCode::VarDecl
        | OpCode::ArgDecl
        | OpCode::Export => {
          let index = self.read(offset);
          offset += 1;
          (
            "----".to_string(),
            format!("{index:02x}"),
            format!("{:?}", self.constants.get(index).to_string()),
          )
        }
        OpCode::Loop => {
          let a = self.read(offset) as u16;
          let b = self.read(offset + 1) as u16;
          offset += 2;
          (
            format!("{:04x}", offset as u16 - ((a << 8) | b)),
            "--".to_string(),
            "-------------------------".to_string(),
          )
        }
        OpCode::Call | OpCode::SetMember | OpCode::GetMember => {
          offset += 1;
          (
            "----".to_string(),
            "--".to_string(),
            "-------------------------".to_string(),
          )
        }
        _ => (
          "----".to_string(),
          "--".to_string(),
          "-------------------------".to_string(),
        ),
      };
      println!(
        "{i:04x} | {:>16} |   {jump_to} |    {index} | {value:>25}",
        format!("{:?}", op)
      );
    }
    println!("===== {name} =====");
  }
}

impl Encode for Chunk {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Chunk as u8];
    {
      encode.push(StructTag::Values as u8);
      for (_, value) in self.constants.enumerate() {
        encode.extend(value.encode()?);
      }
      encode.push(StructTag::EndOfBlock as u8);
    };
    {
      encode.push(StructTag::Code as u8);
      for byte in &self.code {
        let use_byte = match *byte {
          x if x == StructTag::EndOfBlock as u8 => true,
          x if x == StructTag::Byte as u8 => true,
          _ => false,
        };
        if use_byte {
          encode.push(StructTag::Byte as u8);
        }
        encode.push(*byte);
      }
      encode.push(StructTag::EndOfBlock as u8);
    };
    {
      encode.push(StructTag::Lines as u8);
      for line in &self.lines {
        encode.extend(line.encode()?);
      }
      encode.push(StructTag::EndOfBlock as u8);
    };
    Ok(encode)
  }
}
impl Decode for Chunk {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    use crate::util::{OnError as _, OnSome as _};
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Chunk as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un fragmento".to_string())?;
    let constants = {
      vec
        .pop_front()
        .on_some_option(|byte| {
          if byte != StructTag::Values as u8 {
            None
          } else {
            Some(byte)
          }
        })
        .on_error(|_| "Se esperaban valores de un fragmento".to_string())?;
      let mut constants = ValueArray::new();
      loop {
        let byte = vec.front().on_error(|_| "Binario corrupto".to_string())?;
        if *byte == StructTag::EndOfBlock as u8 {
          vec.pop_front(); // EOB
          break;
        }
        constants.write(super::Value::decode(vec)?);
      }
      constants
    };
    let code = {
      vec
        .pop_front()
        .on_some_option(|byte| {
          if byte != StructTag::Code as u8 {
            None
          } else {
            Some(byte)
          }
        })
        .on_error(|_| "Se esperaba codigo de un fragmento".to_string())?;
      let mut code = vec![];
      loop {
        let byte = vec
          .pop_front()
          .on_error(|_| "Binario corrupto".to_string())?;
        if byte == StructTag::EndOfBlock as u8 {
          break;
        }
        let byte = if byte == StructTag::Byte as u8 {
          vec
            .pop_front()
            .on_error(|_| "Binario corrupto".to_string())?
        } else {
          byte
        };
        code.push(byte);
      }
      code
    };
    let lines = {
      vec
        .pop_front()
        .on_some_option(|byte| {
          if byte != StructTag::Lines as u8 {
            None
          } else {
            Some(byte)
          }
        })
        .on_error(|_| "Se esperaban lineas de un fragmento".to_string())?;
      let mut lines = vec![];
      loop {
        let byte = vec.front().on_error(|_| "Binario corrupto".to_string())?;
        if *byte == StructTag::EndOfBlock as u8 {
          vec.pop_front(); // EOB
          break;
        }
        lines.push(usize::decode(vec)?);
      }
      lines
    };
    Ok(Self {
      code,
      lines,
      constants,
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkGroup {
  chunks: Vec<Chunk>,
  aggregate_len: Vec<usize>,
  current: usize,
}
impl ChunkGroup {
  pub fn new_ref() -> MultiRefHash<Self> {
    Self {
      chunks: vec![Chunk::new()],
      aggregate_len: vec![0],
      current: 0,
    }
    .into()
  }
  fn resolve_index(&self, index: usize) -> usize {
    for (i, &agg_len) in self.aggregate_len.iter().enumerate() {
      if index < agg_len {
        return i;
      }
    }
    self.aggregate_len.len() - 1
  }

  pub(self) fn current_chunk_mut(&mut self) -> &mut Chunk {
    &mut self.chunks[self.current]
  }
  pub(self) fn current_chunk(&self) -> &Chunk {
    &self.chunks[self.current]
  }
  pub fn update_aggregate_len(&mut self) {
    self.aggregate_len[self.current] =
      self.prev_aggregate_len() + self.current_chunk_mut().code.len();
  }
  pub fn len(&self) -> usize {
    self.aggregate_len[self.current]
  }
  pub fn get_line(&self, index: usize) -> usize {
    let resolved_index = self.resolve_index(index);
    let base = if resolved_index == 0 {
      0
    } else {
      self.aggregate_len[resolved_index - 1]
    };
    let local_index = index - base;
    *self
      .chunks
      .get(resolved_index)
      .and_then(|chunk| chunk.lines.get(local_index))
      .unwrap_or(&0)
  }

  fn prev_aggregate_len(&self) -> usize {
    if self.current == 0 {
      0
    } else {
      self.aggregate_len[self.current - 1]
    }
  }

  pub fn read_constant(&self, index: u8) -> &super::Value {
    self.current_chunk().constants.get(index)
  }
  pub fn read_var(&mut self, name: String, line: usize) -> u8 {
    if self.current_chunk_mut().constants.len() == u8::MAX {
      self.current += 1;
      self.chunks.push(Chunk::new());
      self.aggregate_len.push(self.prev_aggregate_len());
    }
    let index = self
      .current_chunk_mut()
      .add_constant(super::Value::String(name));
    self.write_buffer(vec![OpCode::GetVar as u8, index], line);
    index
  }
  pub fn make_arg(&mut self, name: String, line: usize) -> u8 {
    if self.current_chunk_mut().constants.len() == u8::MAX {
      self.current += 1;
      self.chunks.push(Chunk::new());
      self.aggregate_len.push(self.prev_aggregate_len());
    }
    let index = self
      .current_chunk_mut()
      .add_constant(super::Value::String(name));
    self.write_buffer(vec![OpCode::ArgDecl as u8, index], line);
    index
  }
  pub fn add_value(&mut self, value: super::Value) -> u8 {
    if self.current_chunk_mut().constants.has_value(&value) {
      self
        .current_chunk_mut()
        .constants
        .get_index(&value)
        .unwrap_or_default()
    } else {
      if self.current_chunk_mut().constants.len() == u8::MAX {
        self.current += 1;
        self.chunks.push(Chunk::new());
        self.aggregate_len.push(self.prev_aggregate_len());
      };
      self.current_chunk_mut().add_constant(value)
    }
  }
  pub fn write_constant(&mut self, value: super::Value, line: usize) -> u8 {
    let index = self.add_value(value);
    self.write_buffer(vec![OpCode::Constant as u8, index], line);
    index
  }
  pub fn write_buffer(&mut self, bytes: Vec<u8>, line: usize) {
    self.current_chunk_mut().write_buffer(bytes, line);
    self.update_aggregate_len();
  }

  pub fn read(&self, index: usize) -> u8 {
    let resolved_index = self.resolve_index(index);
    let base = if resolved_index == 0 {
      0
    } else {
      self.aggregate_len[resolved_index - 1]
    };
    let local_index = index - base;
    self.chunks[resolved_index].read(local_index)
  }
  pub fn write(&mut self, byte: u8, line: usize) {
    self.current_chunk_mut().write(byte, line);
    self.update_aggregate_len();
  }
  pub fn jump(&mut self, code: OpCode) -> usize {
    let v = self.current_chunk_mut().jump(code);
    self.update_aggregate_len();
    v
  }
  pub fn patch_jump(&mut self, offset: usize) -> Result<(), String> {
    let v = self.current_chunk_mut().patch_jump(offset);
    self.update_aggregate_len();
    v
  }
  pub fn add_loop(&mut self, offset: usize) -> Result<(), String> {
    let v = self.current_chunk_mut().add_loop(offset);
    self.update_aggregate_len();
    v
  }
  pub fn _print(&mut self) {
    for (i, chunk) in self.chunks.iter().enumerate() {
      chunk._print(format!("chunk {i}"));
    }
  }
}
impl Default for ChunkGroup {
  fn default() -> Self {
    let mut group = Self {
      chunks: vec![Chunk::new()],
      aggregate_len: vec![0],
      current: 0,
    };
    group.write_constant(super::Value::Never, 0);
    group.write(OpCode::Return as u8, 0);
    group
  }
}
impl Encode for ChunkGroup {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::ChunkGroup as u8];

    for chunk in &self.chunks {
      encode.extend(chunk.encode()?)
    }

    encode.push(StructTag::EndOfBlock as u8);
    Ok(encode)
  }
}
impl Decode for ChunkGroup {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    use crate::util::{OnError as _, OnSome as _};
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::ChunkGroup as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un grupo de fragmentos".to_string())?;
    let mut chunk_group = Self {
      chunks: vec![],
      aggregate_len: vec![0],
      current: 0,
    };
    loop {
      let byte = vec.front().on_error(|_| "Binario corrupto".to_string())?;
      if *byte == StructTag::EndOfBlock as u8 {
        vec.pop_front(); // EOB
        break;
      }
      chunk_group.chunks.push(Chunk::decode(vec)?);
      chunk_group.update_aggregate_len();
    }
    Ok(chunk_group)
  }
}
