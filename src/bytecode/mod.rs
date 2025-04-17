use vm::VM;

use crate::parser::Node;

mod call_stack;
mod compiler;
mod chunk;
mod value;
mod vm;

pub fn main(node: &Node) -> Result<(), String> {
  let mut vm = VM::new(node.into());
  vm.interpret();
  Ok(())
}