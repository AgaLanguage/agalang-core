use std::fmt::format;

use chunk::{Chunk, OpCode};
use vm::VM;

use crate::parser::Node;

mod call_stack;
mod compiler;
mod chunk;
mod value;
mod vm;

pub fn main(node: &Node) -> Result<(), String> {
  let mut vm = VM::new();
  let mut chunk = Chunk::new();
  compiler::node_to_bytes(node, &mut chunk)?;
  vm.interpret(&chunk);
  Ok(())
}