use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use thread::Thread;

use super::cache::Cache;
use super::compiler::{Compiler, OpCode};
use super::stack::{call_stack_to_string, CallFrame, InterpretResult, VarsManager};
use super::value::{Object, Promise, PromiseData, Value};

mod thread;

#[derive(Clone, Debug)]
pub enum BlockingThread {
  Void,
  Module(Rc<RefCell<ModuleThread>>),
  Await(Promise),
}
impl BlockingThread {
  pub fn new() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self::Void))
  }
  pub fn run_instruction(&self) -> InterpretResult {
    match self {
      // nada que ejecutar
      Self::Void => InterpretResult::Ok,
      Self::Module(ref_cell) => ref_cell.borrow().run_instruction(),
      Self::Await(promise) => {
        let data = promise.get_data();
        match data {
          PromiseData::Pending => InterpretResult::Continue,
          // El error pasa a ser del programa, ya que no ha sido tratado
          PromiseData::Err(error) => InterpretResult::RuntimeError(error),
          // La promesa sera desenvolvida en la siguiente instruccion
          PromiseData::Ok(_) => InterpretResult::Ok,
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct ModuleThread {
  path: String,
  sub_module: Rc<RefCell<BlockingThread>>,
  value: Value,
  thread: Rc<RefCell<Thread>>,
  status: InterpretResult,
}
impl ModuleThread {
  pub fn as_value(self) -> Value {
    self.value
  }
  pub fn run_instruction(&self) -> InterpretResult {
    let sub_module = self.sub_module.borrow().clone();
    if matches!(sub_module, BlockingThread::Void) {
      return self.thread.borrow_mut().run_instruction();
    }
    let data = sub_module.run_instruction();
    if matches!(data, InterpretResult::Ok) {
      *self.sub_module.borrow_mut() = BlockingThread::Void;
      return InterpretResult::Continue;
    }
    // cualquier error afecta directamente al programa
    data
  }
  pub fn push_call(&self, frame: CallFrame) {
    let sub_module = self.sub_module.borrow().clone();
    if let BlockingThread::Module(sub_module) = sub_module {
      return sub_module.borrow_mut().push_call(frame);
    }
    self.thread.borrow_mut().push_call(frame);
  }
}

#[derive(Clone, Debug)]
pub struct AsyncThread {
  promise: Promise,
  thread: Rc<RefCell<Thread>>,
}
impl AsyncThread {
  pub fn from_frame(parent: &Thread, frame: CallFrame) -> (Self, Promise) {
    let (thread, promise) = Self::new(parent.get_vm());
    thread.push_call(frame);
    (thread, promise)
  }
  pub fn new(vm: Rc<RefCell<VM>>) -> (Self, Promise) {
    let pr = Promise::new();
    let promise = pr.clone();
    (
      Self {
        promise,
        thread: Thread::new(vm),
      },
      pr,
    )
  }
  pub fn push_call(&self, frame: CallFrame) {
    self.thread.borrow_mut().push_call(frame)
  }
  pub fn run_instruction(&self) -> InterpretResult {
    let is_return = self.thread.borrow_mut().peek() == OpCode::OpReturn;
    if is_return {
      let value = self.thread.borrow_mut().pop();
      self.promise.set_value(value);
      return InterpretResult::Ok;
    }
    let result = self.thread.borrow_mut().run_instruction();
    match result {
      InterpretResult::RuntimeError(err) => {
        self.promise.set_err(err);
        // Este es un error de la promesa, no de el programa
        InterpretResult::Ok
      }
      result => result,
    }
  }
}

#[derive(Clone, Debug)]
pub struct VM {
  globals: Rc<RefCell<VarsManager>>,
  pub cache: Cache,
  module: Rc<RefCell<Option<ModuleThread>>>,
  sub_threads: Rc<RefCell<Vec<AsyncThread>>>,
}

impl VM {
  pub fn push_sub_thread(&mut self, thread: AsyncThread) {
    self.sub_threads.borrow_mut().push(thread);
  }
  pub fn new(compiler: Compiler) -> Rc<RefCell<Self>> {
    //let compiler = {let mut compiler = compiler;compiler.function.chunk().print();compiler};
    let globals = Rc::new(RefCell::new(VarsManager::get_global()));

    let main: Rc<RefCell<Option<ModuleThread>>> = Default::default();
    let vm: Rc<RefCell<VM>> = Rc::new(RefCell::new(Self {
      globals: globals.clone(),
      sub_threads: Default::default(),
      cache: Default::default(),
      module: main.clone(),
    }));
    let thread = Thread::new(vm.clone());
    let module = ModuleThread {
      sub_module: BlockingThread::new(),
      path: (&compiler).path.clone(),
      thread: thread.clone(),
      value: Value::Object(Object::Map(HashMap::new().into(), HashMap::new().into())),
      status: InterpretResult::Continue,
    };
    *main.borrow_mut() = Some(module);
    thread.borrow_mut().push_call(CallFrame::new_compiler(
      compiler,
      vm.borrow().globals.clone(),
    ));
    vm
  }
  pub fn as_value(&self) -> Value {
    self.module.borrow().clone().unwrap().as_value()
  }
  fn run_instruction(&mut self) -> InterpretResult {
    let mut sub_threads = vec![];
    for thread in &mut self.sub_threads.borrow().iter() {
      let data = thread.run_instruction();
      if matches!(data, InterpretResult::Continue) {
        sub_threads.push(thread.clone());
      }
    }
    *self.sub_threads.borrow_mut() = sub_threads;

    let module = self.module.borrow_mut().clone().unwrap();
    module.run_instruction()
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
    let binding = self.module.borrow();
    let module = binding.as_ref().unwrap();
    let mut thread = module.thread.borrow_mut();
    match &result {
      InterpretResult::RuntimeError(e) => thread.clone().runtime_error(&format!(
        "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
        e,
        call_stack_to_string(&thread.get_calls())
      )),
      InterpretResult::CompileError(e) => {
        thread.runtime_error(&format!("Error en compilacion\n\t{}", e))
      }
      _ => {}
    };
    let binding = thread.clone();
    let stack = binding.get_stack();
    if stack.len() != 0 {
      thread.runtime_error(&format!("Error de pila no vacia | {stack:?}"));
    }
    result
  }
  pub fn resolve(
    this: Rc<RefCell<Self>>,
    path: &str,
    vars: Rc<RefCell<VarsManager>>,
  ) -> ModuleThread {
    let mut result = ModuleThread {
      value: Value::Object(Object::Map(HashMap::new().into(), HashMap::new().into())),
      path: path.to_string(),
      status: InterpretResult::Continue,
      sub_module: Rc::new(RefCell::new(BlockingThread::Void)),
      thread: Thread::new(this.clone()),
    };
    result.thread.borrow().set_module(result.clone());

    let file = match crate::code(path) {
      None => {
        result.status = InterpretResult::NativeError;
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
        result.status = InterpretResult::NativeError;
        return result;
      }
      Ok(value) => value,
    };
    result.push_call(CallFrame::new_compiler(ast.into(), vars));
    result
  }
  pub fn clear_stack(&self) {
    self
      .module
      .borrow()
      .clone()
      .unwrap()
      .thread
      .borrow_mut()
      .clear_stack();
  }
}
