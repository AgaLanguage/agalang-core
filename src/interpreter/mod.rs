mod cache;
mod libs;
mod stack;
mod vm;
pub mod proto;
pub use vm::Thread;
pub use stack::VarsManager;

pub fn interpret(compiler: crate::compiler::Compiler) -> Result<crate::compiler::Value, ()>{
    let binding = vm::VM::new(compiler);
  let mut vm = binding.borrow().clone();
  match vm.interpret() {
    stack::InterpretResult::Ok => {}
    _ => {
      return Err(());
    }
  }
  Ok(vm.clone().as_value())
}