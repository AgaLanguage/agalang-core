use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

pub use thread::Thread;
use thread::{AsyncThread, ModuleThread};

use super::cache::Cache;
use super::compiler::Compiler;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult, VarsManager};
use super::value::Value;

mod thread;
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
      let data = async_thread.borrow().run_instruction_as_module();
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
      InterpretResult::RuntimeError(e) => thread.borrow_mut().runtime_error(&format!(
        "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
        e,
        call_stack_to_string(thread.borrow_mut().get_calls())
      )),
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

    let file = match crate::code(path) {
      None => {
        result.borrow_mut().set_status(InterpretResult::NativeError);
        return result;
      }
      Some(value) => value,
    };

    let ref ast = match crate::parser::Parser::new(&file, &path).produce_ast() {
      Err(a) => {
        crate::parser::print_error(crate::parser::error_to_string(
          &crate::parser::ErrorNames::SyntaxError,
          crate::parser::node_error(&a, &file),
        ));
        result.borrow_mut().set_status(InterpretResult::NativeError);
        return result;
      }
      Ok(value) => value,
    };
    result
      .borrow()
      .push_call(CallFrame::new_compiler(ast.into(), vars));
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
