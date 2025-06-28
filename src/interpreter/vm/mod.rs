use std::fmt::Debug;
use std::path::Path;

use super::cache::Cache;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult};
use super::VarsManager;
use crate::compiler::{Compiler, Value};
use crate::MultiRefHash;

mod thread;
pub use thread::{AsyncThread, ModuleThread, Thread};

mod process;

#[derive(Clone, Debug)]
pub struct VM {
  pub cache: Cache,
  globals: MultiRefHash<VarsManager>,
  process_manager: MultiRefHash<process::ProcessManager>,
}

impl VM {
  pub fn new(compiler: Compiler) -> MultiRefHash<Self> {
    //let compiler = {let mut compiler = compiler;compiler.function.chunk().print();compiler};
    let globals: MultiRefHash<VarsManager> = VarsManager::get_global().into();

    let module = ModuleThread::new(&compiler.path);
    let vm: MultiRefHash<VM> = Self {
      globals,
      cache: Default::default(),
      process_manager: process::ProcessManager::new(module.clone()).into(),
    }
    .into();
    module.write().set_vm(vm.clone());
    module
      .read()
      .push_call(CallFrame::new_compiler(compiler, vm.read().globals.clone()));
    vm
  }
  pub fn as_value(&self) -> Value {
    self.process_manager.read().as_value()
  }
  pub fn get_process_manager(&self) -> MultiRefHash<process::ProcessManager> {
    self.process_manager.clone()
  }
  pub fn run(&mut self) -> InterpretResult {
    loop {
      let data = self.process_manager.read().run_instruction();
      match &data {
        InterpretResult::Continue => continue,
        InterpretResult::Ok => {}
        _error => {
          self.clear_stack();
        }
      }
      return data;
    }
  }
  pub fn interpret(&mut self) -> InterpretResult {
    let result = self.run();
    let thread = self.process_manager.read().get_root_thread();
    match &result {
      InterpretResult::RuntimeError(e) => {
        let calls = thread.read().get_calls().clone();
        thread.write().runtime_error(&format!(
          "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
          e,
          call_stack_to_string(&calls)
        ))
      }
      InterpretResult::CompileError(e) => thread
        .write()
        .runtime_error(&format!("Error en compilacion\n\t{}", e,)),
      _ => {}
    };
    let stack = thread.read().get_stack().clone();
    if !stack.is_empty() {
      thread
        .write()
        .runtime_error(&format!("Error de pila no vacia | {stack:?}"));
    }
    result
  }
  pub fn resolve(
    this: MultiRefHash<Self>,
    path: &str,
    vars: MultiRefHash<VarsManager>,
  ) -> MultiRefHash<ModuleThread> {
    let result = ModuleThread::new(path);
    result.write().set_vm(this);

    let compiler = match crate::compile(Path::new(path)) {
      Ok((compiler, _)) => compiler,
      Err(e) => {
        result.write().set_status(InterpretResult::CompileError(e));
        return result;
      }
    };

    result
      .read()
      .push_call(CallFrame::new_compiler(compiler, vars));
    result
  }
  pub fn clear_stack(&self) {
    self
      .process_manager
      .read()
      .get_root_thread()
      .write()
      .clear_stack();
  }
}
