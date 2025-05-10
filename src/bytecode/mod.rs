use vm::VM;

use crate::{parser::Node, value::Value};

mod compiler;
mod vm;

pub use compiler::ChunkGroup;

pub fn main(node: &Node) -> Result<Value, String> {
  let mut vm = VM::new(node.into());
  vm.interpret();
  Ok(vm.as_value())
}
