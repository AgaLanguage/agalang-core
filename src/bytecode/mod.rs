use stack::InterpretResult;
use value::Value;
use vm::VM;

use crate::parser::Node;

mod cache;
mod compiler;
mod libs;
mod proto;
mod stack;
mod value;
mod vm;

pub use cache::DataCache;
pub use compiler::ChunkGroup;

pub fn main(node: &Node) -> Result<Value, ()> {
  let binding = VM::new(node.into());
  let mut vm = binding.borrow().clone().unwrap();
  match vm.interpret() {
    InterpretResult::Ok => {}
    _ => {
      return Err(());
    }
  }
  Ok(vm.clone().as_value())
}
