use super::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
  // Const
  OpConstant,
  OpConstantLong,
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
      x if x == OpCode::OpConstantLong as u8 => OpCode::OpConstantLong,
      x if x == OpCode::OpAdd as u8 => OpCode::OpAdd,
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

#[derive(Debug, Clone)]
pub struct Chunk {
  code: Vec<u8>,
  pub lines: Vec<usize>,
  pub constants: ValueArray,
  other_chunk: Option<Box<Chunk>>,
  pub last_code: usize,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      lines: Vec::new(),
      constants: ValueArray::new(),
      other_chunk: None,
      last_code: 0,
    }
  }
  pub fn read(&self, index: usize) -> u8 {
    if let Some(chunk) = &self.other_chunk {
      if chunk.last_code >= index {
        chunk.read(index)
      } else {
        self.code[index - chunk.last_code]
      }
    } else {
      self.code[index]
    }
  }
  fn overwrite(&mut self, index: usize, byte: u8) {
    if let Some(chunk) = &mut self.other_chunk {
      if chunk.last_code > index {
        chunk.overwrite(index, byte);
      } else {
        self.code[index - chunk.last_code] = byte;
      }
    } else {
      self.code[index] = byte;
    }
  }
  pub fn write(&mut self, byte: u8, line: usize) {
    self.code.push(byte);
    self.lines.push(line);
    self.last_code = self.code.len();
  }
  pub fn write_buffer(&mut self, bytes: Vec<u8>, line: usize) {
    for byte in bytes {
      self.write(byte, line);
    }
  }
  fn add_constant(&mut self, value: Value) -> usize {
    self.constants.write(value);
    self.constants.len() - 1
  }
  pub fn make_constant(&mut self, value: Value) -> u8 {
    if self.constants.len() == 0xFF {
      let chunk = self.clone();
      self.clear();
      self.other_chunk = Some(Box::new(chunk));
    }
    self.add_constant(value) as u8
  }
  pub fn add_loop(&mut self, loop_start: usize) -> Result<(), String> {
    self.write(OpCode::OpLoop as u8, self.last_code);
    
    let offset = self.last_code - loop_start + 2;
    if offset > u16::MAX.into() {
      return Err(format!("Longitud muy alta"));
    }
    self.write( ((offset >> 8) & 0xff) as u8, self.last_code);
    self.write( (offset & 0xff) as u8, self.last_code);
    Ok(())
  }
  pub fn jump(&mut self, code: OpCode) -> usize {
    self.write_buffer(vec![code as u8, 0xFF, 0xFF], self.last_code);
    self.last_code - 2
  }
  pub fn patch_jump(&mut self, offset: usize) -> Result<(), String> {
    let jump = self.last_code - offset - 2;
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
}
