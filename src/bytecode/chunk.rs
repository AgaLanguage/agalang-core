use super::value::{Value, ValueArray};

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
  OpAsBoolean,
  OpAsString,
  OpCall,
  OpArgDecl,
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
  // Control
  OpPop,
  OpNewLocals,
  OpRemoveLocals,
  OpJumpIfFalse,
  OpJump,
  OpReturn,
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
      x if x == OpCode::OpConstant as u8 => OpCode::OpConstant,
      x if x == OpCode::OpCall as u8 => OpCode::OpCall,
      x if x == OpCode::OpAdd as u8 => OpCode::OpAdd,
      x if x == OpCode::OpArgDecl as u8 => OpCode::OpArgDecl,
      x if x == OpCode::OpSubtract as u8 => OpCode::OpSubtract,
      x if x == OpCode::OpMultiply as u8 => OpCode::OpMultiply,
      x if x == OpCode::OpDivide as u8 => OpCode::OpDivide,
      x if x == OpCode::OpNegate as u8 => OpCode::OpNegate,
      x if x == OpCode::OpNot as u8 => OpCode::OpNot,
      x if x == OpCode::OpAsBoolean as u8 => OpCode::OpAsBoolean,
      x if x == OpCode::OpAsString as u8 => OpCode::OpAsString,
      x if x == OpCode::OpGreaterThan as u8 => OpCode::OpGreaterThan,
      x if x == OpCode::OpLessThan as u8 => OpCode::OpLessThan,
      x if x == OpCode::OpEquals as u8 => OpCode::OpEquals,
      x if x == OpCode::OpConsoleOut as u8 => OpCode::OpConsoleOut,
      x if x == OpCode::OpGetVar as u8 => OpCode::OpGetVar,
      x if x == OpCode::OpSetVar as u8 => OpCode::OpSetVar,
      x if x == OpCode::OpVarDecl as u8 => OpCode::OpVarDecl,
      x if x == OpCode::OpConstDecl as u8 => OpCode::OpConstDecl,
      x if x == OpCode::OpPop as u8 => OpCode::OpPop,
      x if x == OpCode::OpAnd as u8 => OpCode::OpAnd,
      x if x == OpCode::OpOr as u8 => OpCode::OpOr,
      x if x == OpCode::OpLoop as u8 => OpCode::OpLoop,
      x if x == OpCode::OpNewLocals as u8 => OpCode::OpNewLocals,
      x if x == OpCode::OpRemoveLocals as u8 => OpCode::OpRemoveLocals,
      x if x == OpCode::OpJumpIfFalse as u8 => OpCode::OpJumpIfFalse,
      x if x == OpCode::OpJump as u8 => OpCode::OpJump,
      x if x == OpCode::OpReturn as u8 => OpCode::OpReturn,

      x => OpCode::OpNull,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Chunk {
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
  fn clear(&mut self) {
    self.code = vec![];
    self.lines = vec![];
    self.constants = ValueArray::new();
  }
  fn print(&self, name: String) {
    println!("===== {name} =====");
    println!("Byte | Operation        | JumpTo | Index | Value",);
    let mut offset = 0;
    while offset < self.code.len() {
      let i = offset;
      let op = OpCode::from(self.code[offset]);
      offset += 1;
      print!("{:04x} | {:>16} ", i, format!("{:?}", op));
      let JumpTo = if op == OpCode::OpJump || op == OpCode::OpJumpIfFalse {
        let a = self.read(offset) as u16;
        let b = self.read(offset + 1) as u16;
        offset += 2;
        format!("{:04x}", (a << 8) | b)
      } else {
        "----".into()
      };
      print!("|   {JumpTo} ");
      let (index, value) = if op == OpCode::OpConstant || op == OpCode::OpGetVar {
        let index = self.read(offset);
        offset += 1;
        (
          format!("{index:02x}"),
          format!("{:?}", self.constants.get(index)),
        )
      } else if op == OpCode::OpConstDecl || op == OpCode::OpVarDecl {
        let index = self.read(offset);
        offset += 1;
        (
          format!("{index:02x}"),
          format!("{:?}", self.constants.get(index)),
        )
      } else {
        ("--".into(), "-------------------------".into())
      };
      println!("|    {index} | {value:>25}")
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
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
      .add_constant(Value::Object(name.as_str().into()));
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
      .add_constant(Value::Object(name.as_str().into()));
    self.write_buffer(vec![OpCode::OpArgDecl as u8, index], line);
    index
  }
  pub fn make_constant(&mut self, value: Value) -> u8 {
    let line = self.current_chunk_mut().lines.last().unwrap_or(&0) + 1;
    self.write_constant(value, line)
  }
  pub fn write_constant(&mut self, value: Value, line: usize) -> u8 {
    if self.current_chunk_mut().constants.len() >= u8::MAX {
      self.current += 1;
      self.chunks.push(Chunk::new());
      self.aggregate_len.push(self.prev_aggregate_len());
    }
    let index = self.current_chunk_mut().add_constant(value);
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
