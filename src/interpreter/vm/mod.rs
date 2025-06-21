use std::cell::RefCell;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

use super::cache::Cache;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult};
use super::VarsManager;
use crate::compiler::{Compiler, Value};

mod thread;
use thread::ModuleThread;
pub use thread::Thread;

mod process;

#[derive(Clone, Debug)]
pub struct VM {
  pub cache: Cache,
  globals: Rc<RefCell<VarsManager>>,
  process_manager: Rc<RefCell<process::ProcessManager>>,
}

impl VM {
  pub fn new(compiler: Compiler) -> Rc<RefCell<Self>> {
    //let compiler = {let mut compiler = compiler;compiler.function.chunk().print();compiler};
    let globals = Rc::new(RefCell::new(VarsManager::get_global()));

    let module = ModuleThread::new(&compiler.path);
    let vm: Rc<RefCell<VM>> = Rc::new(RefCell::new(Self {
      globals: globals.clone(),
      cache: Default::default(),
      process_manager: Rc::new(RefCell::new(process::ProcessManager::new(module.clone()))),
    }));
    module.borrow_mut().set_vm(vm.clone());
    module.borrow().push_call(CallFrame::new_compiler(
      compiler,
      vm.borrow().globals.clone(),
    ));
    vm
  }
  pub fn as_value(&self) -> Value {
    self.process_manager.borrow().as_value()
  }
  pub fn get_process_manager(&self) -> Rc<RefCell<process::ProcessManager>> {
    self.process_manager.clone()
  }
  pub fn run(&mut self) -> InterpretResult {
    loop {
      let data = self.process_manager.borrow().run_instruction();
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
    let thread = self.process_manager.borrow().get_root_thread();
    match &result {
      InterpretResult::RuntimeError(e) => {
        let calls = thread.borrow().get_calls().clone();
        thread.borrow_mut().runtime_error(&format!(
          "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
          e,
          call_stack_to_string(&calls)
        ))
      }
      InterpretResult::CompileError(e) => thread
        .borrow_mut()
        .runtime_error(&format!("Error en compilacion\n\t{}", e,)),
      _ => {}
    };
    let stack = thread.borrow().get_stack().clone();
    if stack.len() != 0 {
      thread
        .borrow_mut()
        .runtime_error(&format!("Error de pila no vacia | {stack:?}"));
    }
    result
  }
  pub fn resolve(
    this: Rc<RefCell<Self>>,
    path: &str,
    vars: Rc<RefCell<VarsManager>>,
  ) -> Rc<RefCell<ModuleThread>> {
    let result = ModuleThread::new(path);
    result.borrow_mut().set_vm(this);

    let compiler = match crate::compile(Path::new(path)) {
      Ok((compiler, _)) => compiler,
      Err(e) => {
        result
          .borrow_mut()
          .set_status(InterpretResult::CompileError(e));
        return result;
      }
    };

    result
      .borrow()
      .push_call(CallFrame::new_compiler(compiler, vars));
    result
  }
  pub fn clear_stack(&self) {
    self
      .process_manager
      .borrow()
      .get_root_thread()
      .borrow_mut()
      .clear_stack();
  }
}
