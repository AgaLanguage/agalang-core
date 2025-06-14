use std::cell::RefCell;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

use super::cache::Cache;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult};
use super::VarsManager;
use crate::compiler::{Compiler, Value};

mod thread;
pub use thread::Thread;
use thread::{AsyncThread, ModuleThread};
#[derive(Clone, Debug)]
pub struct VM {
  globals: Rc<RefCell<VarsManager>>,
  pub cache: Cache,
  module: Rc<RefCell<ModuleThread>>,
  sub_threads: Rc<RefCell<Vec<Rc<RefCell<AsyncThread>>>>>,
}

impl VM {
  pub fn push_sub_thread(&mut self, thread: Rc<RefCell<AsyncThread>>) {
    self.sub_threads.borrow_mut().push(thread);
  }
  pub fn new(compiler: Compiler) -> Rc<RefCell<Self>> {
    //let compiler = {let mut compiler = compiler;compiler.function.chunk().print();compiler};
    let globals = Rc::new(RefCell::new(VarsManager::get_global()));

    let module = ModuleThread::new(&compiler.path);
    let vm: Rc<RefCell<VM>> = Rc::new(RefCell::new(Self {
      globals: globals.clone(),
      sub_threads: Default::default(),
      cache: Default::default(),
      module: module.clone(),
    }));
    module.borrow_mut().set_vm(vm.clone());
    module.borrow().push_call(CallFrame::new_compiler(
      compiler,
      vm.borrow().globals.clone(),
    ));
    vm
  }
  pub fn as_value(&self) -> Value {
    self.module.borrow().clone().as_value()
  }
  fn run_instruction(&self) -> InterpretResult {
    let mut sub_threads = vec![];
    for async_thread in &mut self.sub_threads.borrow().iter() {
      let data = async_thread.borrow().simple_run_instruction(true);
      if matches!(data, InterpretResult::Continue) {
        sub_threads.push(async_thread.clone());
      }
    }
    *self.sub_threads.borrow_mut() = sub_threads;

    self.module.borrow().run_instruction()
  }
  pub fn run(&mut self) -> InterpretResult {
    loop {
      let data = self.run_instruction();
      match data {
        InterpretResult::Ok => {
          if self.sub_threads.borrow().is_empty() {
            return data;
          }
        }
        InterpretResult::Continue => {}
        error => {
          self.clear_stack();
          return error;
        }
      }
    }
  }
  pub fn interpret(&mut self) -> InterpretResult {
    let result = self.run();
    let thread = self.module.borrow().get_async().borrow().get_thread();
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
      .module
      .borrow()
      .get_async()
      .borrow()
      .get_thread()
      .borrow_mut()
      .clear_stack();
  }
}
