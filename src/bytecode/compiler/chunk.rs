use crate::bytecode::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
  // Const
  OpConstant,
  // Math
  OpAdd,
  OpSubtract,
  OpMultiply,
  OpDivide,
  OpNegate,
  // Expr
  OpNot,
  OpApproximate,
  OpAsBoolean,
  OpAsString,
  OpCall,
  OpArgDecl,
  OpGetMember,
  OpSetMember,
  // Binary
  OpAnd,
  OpOr,
  OpGreaterThan,
  OpLessThan,
  OpEquals,
  // Statement
  OpConsoleOut,
  OpVarDecl,
  OpConstDecl,
  OpGetVar,
  OpSetVar,
  OpLoop,
  OpImport,
  OpExport,
  // Control
  OpPop,
  OpNewLocals,
  OpRemoveLocals,
  OpJumpIfFalse,
  OpJump,
  OpReturn,
  OpBreak,
  OpContinue,
  OpCopy, // Para duplicar el ultimo valor en el stack (obtener el padre de un objeto)
  // Invalid
  OpNull,
}
impl From<&u8> for OpCode {
  fn from(value: &u8) -> Self {
    (*value).into()
  }
}
impl From<u8> for OpCode {
  fn from(value: u8) -> Self {
    match value {
      x if x == Self::OpApproximate as u8 => Self::OpApproximate,
      x if x == Self::OpGetMember as u8 => Self::OpGetMember,
      x if x == Self::OpSetMember as u8 => Self::OpSetMember,
      x if x == Self::OpConstant as u8 => Self::OpConstant,
      x if x == Self::OpCall as u8 => Self::OpCall,
      x if x == Self::OpAdd as u8 => Self::OpAdd,
      x if x == Self::OpArgDecl as u8 => Self::OpArgDecl,
      x if x == Self::OpSubtract as u8 => Self::OpSubtract,
      x if x == Self::OpMultiply as u8 => Self::OpMultiply,
      x if x == Self::OpDivide as u8 => Self::OpDivide,
      x if x == Self::OpNegate as u8 => Self::OpNegate,
      x if x == Self::OpNot as u8 => Self::OpNot,
      x if x == Self::OpAsBoolean as u8 => Self::OpAsBoolean,
      x if x == Self::OpAsString as u8 => Self::OpAsString,
      x if x == Self::OpGreaterThan as u8 => Self::OpGreaterThan,
      x if x == Self::OpLessThan as u8 => Self::OpLessThan,
      x if x == Self::OpEquals as u8 => Self::OpEquals,
      x if x == Self::OpConsoleOut as u8 => Self::OpConsoleOut,
      x if x == Self::OpGetVar as u8 => Self::OpGetVar,
      x if x == Self::OpSetVar as u8 => Self::OpSetVar,
      x if x == Self::OpVarDecl as u8 => Self::OpVarDecl,
      x if x == Self::OpConstDecl as u8 => Self::OpConstDecl,
      x if x == Self::OpPop as u8 => Self::OpPop,
      x if x == Self::OpAnd as u8 => Self::OpAnd,
      x if x == Self::OpOr as u8 => Self::OpOr,
      x if x == Self::OpLoop as u8 => Self::OpLoop,
      x if x == Self::OpNewLocals as u8 => Self::OpNewLocals,
      x if x == Self::OpRemoveLocals as u8 => Self::OpRemoveLocals,
      x if x == Self::OpJumpIfFalse as u8 => Self::OpJumpIfFalse,
      x if x == Self::OpJump as u8 => Self::OpJump,
      x if x == Self::OpReturn as u8 => Self::OpReturn,
      x if x == Self::OpCopy as u8 => Self::OpCopy,
      x if x == Self::OpImport as u8 => Self::OpImport,
      x if x == Self::OpExport as u8 => Self::OpExport,

      _ => Self::OpNull,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub lines: Vec<usize>,
  pub constants: ValueArray,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      lines: Vec::new(),
      constants: ValueArray::new(),
    }
  }
  pub fn read(&self, index: usize) -> u8 {
    return self.code[index];
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
  fn add_constant(&mut self, value: Value) -> u8 {
    if self.constants.has_value(&value) {
      return self.constants.get_index(&value).unwrap_or(0);
    }
    self.constants.write(value);
    self.constants.len() - 1
  }
  pub fn add_loop(&mut self, loop_start: usize) -> Result<(), String> {
    self.write(OpCode::OpLoop as u8, self.code.len());

    let offset = self.code.len() - loop_start + 2;
    if offset > u16::MAX.into() {
      return Err(format!("Longitud muy alta"));
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
    let jump = self.code.len() - offset - 2;
    if jump > u16::MAX.into() {
      return Err(format!("Longitud muy alta"));
    }
    self.overwrite(offset, ((jump >> 8) & 0xff) as u8);
    self.overwrite(offset + 1, (jump & 0xff) as u8);
    Ok(())
  }
  fn print(&self, name: String) {
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
        OpCode::OpJump | OpCode::OpJumpIfFalse => {
          let a = self.read(offset) as u16;
          let b = self.read(offset + 1) as u16;
          offset += 2;
          (format!("{:04x}", (a << 8) | b), "--".into(), "-------------------------".into())
        }
        OpCode::OpConstant | OpCode::OpGetVar | OpCode::OpConstDecl | OpCode::OpVarDecl | OpCode::OpArgDecl | OpCode::OpExport => {
          let index = self.read(offset);
          offset += 1;
          (
            "----".into(),
            format!("{index:02x}"),
            format!("{:?}", self.constants.get(index).as_string()),
          )
        }
        OpCode::OpLoop => {
          let a = self.read(offset) as u16;
          let b = self.read(offset + 1) as u16;
          offset += 2;
          (format!("{:04x}", offset as u16 - ((a << 8) | b)), "--".into(), "-------------------------".into())
        }
        OpCode::OpCall | OpCode::OpSetMember | OpCode::OpGetMember => {
          offset += 1;
          (
            "----".into(),
            "--".into(),
            "-------------------------".into(),
          )
        }
        _ => {
          ("----".into(), "--".into(), "-------------------------".into())
        }
      };
      println!("{i:04x} | {:>16} |   {jump_to} |    {index} | {value:>25}", format!("{:?}", op));
    }
    println!("===== {name} =====");
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkGroup {
  chunks: Vec<Chunk>,
  aggregate_len: Vec<usize>,
  current: usize,
}
impl ChunkGroup {
  pub fn new() -> Self {
    Self {
      chunks: vec![Chunk::new()],
      aggregate_len: vec![0],
      current: 0,
    }
  }
  fn resolve_index(&self, index: usize) -> usize {
    for (i, &agg_len) in self.aggregate_len.iter().enumerate() {
      if index < agg_len {
        return i;
      }
    }
    self.aggregate_len.len() - 1
  }

  pub fn current_chunk_mut(&mut self) -> &mut Chunk {
    &mut self.chunks[self.current]
  }
  pub fn current_chunk(&self) -> &Chunk {
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
      .unwrap_or(&0) // Si quieres devolver Result o Option, se puede cambiar.
  }

  fn prev_aggregate_len(&self) -> usize {
    if self.current == 0 {
      0
    } else {
      self.aggregate_len[self.current - 1]
    }
  }

  pub fn read_constant(&self, index: u8) -> &Value {
    self.current_chunk().constants.get(index)
  }
  pub fn read_var(&mut self, name: String, line: usize) -> u8 {
    if self.current_chunk_mut().constants.len() >= u8::MAX {
      self.current += 1;
      self.chunks.push(Chunk::new());
      self.aggregate_len.push(self.prev_aggregate_len());
    }
    let index = self
      .current_chunk_mut()
      .add_constant(Value::String(name.as_str().into()));
    self.write_buffer(vec![OpCode::OpGetVar as u8, index], line);
    index
  }
  pub fn make_arg(&mut self, name: String, line: usize) -> u8 {
    if self.current_chunk_mut().constants.len() >= u8::MAX {
      self.current += 1;
      self.chunks.push(Chunk::new());
      self.aggregate_len.push(self.prev_aggregate_len());
    }
    let index = self
      .current_chunk_mut()
      .add_constant(Value::String(name.as_str().into()));
    self.write_buffer(vec![OpCode::OpArgDecl as u8, index], line);
    index
  }
  pub fn write_constant(&mut self, value: Value, line: usize) -> u8 {
    let index = if self.current_chunk_mut().constants.has_value(&value) {
      self.current_chunk_mut().constants.get_index(&value).unwrap_or_default()
    }else{
      if self.current_chunk_mut().constants.len() >= u8::MAX {
        self.current += 1;
        self.chunks.push(Chunk::new());
        self.aggregate_len.push(self.prev_aggregate_len());
      };
      self.current_chunk_mut().add_constant(value)
    };
    self.write_buffer(vec![OpCode::OpConstant as u8, index], line);
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
  pub fn print(&mut self) {
    for (i, chunk) in self.chunks.iter().enumerate() {
      chunk.print(format!("chunk {i}"));
    }
  }
}
impl Default for ChunkGroup {
  fn default() -> Self {
    let mut group = Self::new();
    group.write_constant(Value::Never, 0);
    group.write(OpCode::OpReturn as u8, 0);
    group
  }
}