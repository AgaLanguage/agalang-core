mod cache;
mod libs;
pub mod proto;
mod stack;
mod vm;
pub use stack::VarsManager;
pub use vm::{ModuleThread, Thread};

pub fn interpret(compiler: crate::compiler::Compiler) -> Result<crate::compiler::Value, ()> {
  let vm = vm::VM::new(compiler);
  match vm.read().interpret() {
    stack::InterpretResult::Ok => {}
    _ => Err(())?,
  }
  let value = vm.read().as_value();
  Ok(value)
}
