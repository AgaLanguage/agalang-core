use vm::VM;

use crate::parser::Node;

mod chunk;
mod compiler;
mod vm;

pub use chunk::ChunkGroup;

pub fn main(node: &Node) -> Result<(), String> {
  let mut vm = VM::new(node.into());
  vm.interpret();
  Ok(())
}
