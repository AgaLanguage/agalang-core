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

pub fn main(node: &Node) -> Result<Value, String> {
  let mut vm = VM::new(node.into());
  vm.interpret();
  Ok(vm.as_value())
}
