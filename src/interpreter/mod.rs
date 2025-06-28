mod cache;
mod libs;
pub mod proto;
mod stack;
mod vm;
pub use stack::VarsManager;
pub use vm::{ModuleThread, Thread};

pub fn interpret(compiler: crate::compiler::Compiler) -> Result<crate::compiler::Value, ()> {
  let binding = vm::VM::new(compiler);
  let mut vm = binding.read().clone();
  match vm.interpret() {
    stack::InterpretResult::Ok => {}
    _ => {
      Err(())?
    }
  }
  Ok(vm.clone().as_value())
}
