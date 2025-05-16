use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use thread::Thread;

use super::cache::Cache;
use super::compiler::Compiler;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult, VarsManager};
use super::value::{Object, Value};

mod thread;

#[derive(Clone)]
pub struct ModuleThread {
  path: String,
  sub_module: Rc<RefCell<Option<Rc<RefCell<ModuleThread>>>>>,
  module: Value,
  thread: Rc<RefCell<Thread>>,
  status: InterpretResult
}
impl ModuleThread {
  pub fn as_value(self) -> Value {
    self.module
  }
  pub fn run_instruction(&mut self) -> InterpretResult {
    let sub_module = self.sub_module.borrow_mut().clone();
    if let Some(sub_module) = sub_module {
      let data = sub_module.borrow_mut().run_instruction();
      if matches!(data, InterpretResult::Ok) {
        *self.sub_module.borrow_mut() = None;
        return InterpretResult::Continue;
      }
      data
    } else {
      self.thread.borrow_mut().run_instruction()
    }
  }
}

#[derive(Clone)]
pub struct VM {
  globals: Rc<RefCell<VarsManager>>,
  pub cache: Cache,
  main: Rc<RefCell<Option<ModuleThread>>>,
  sub_threads: Vec<Thread>,
}

impl VM {
  pub fn new(compiler: Compiler) -> Rc<RefCell<Option<Self>>> {
    let globals = Rc::new(RefCell::new(VarsManager::get_global()));

    let vm = Rc::new(RefCell::new(Some(Self {
      globals: globals.clone(),
      sub_threads: vec![],
      cache: Cache::new(),
      main: Rc::new(RefCell::new(None)),
    })));

    let thread = Rc::new(RefCell::new(Thread::new(vm.clone(), None)));
    
    let main = vm.borrow().as_ref().unwrap().main.clone();
    let module = 
    Some(ModuleThread {
      sub_module: Rc::new(RefCell::new(None)),
      path: (&compiler).path.clone(),
      thread: thread.clone(),
      module: Value::Object(Object::Map(HashMap::new().into(), HashMap::new().into())),
      status: InterpretResult::Continue
    });
    *main.borrow_mut() = module.clone();
    thread.borrow_mut().module.replace(module);
    thread.borrow_mut().call_stack.push(CallFrame::new_compiler(compiler, vm.borrow().as_ref().unwrap().globals.clone()));
    vm
  }
  pub fn as_value(self) -> Value {
    self.main.borrow().as_ref().unwrap().clone().as_value()
  }
  fn run_instruction(&mut self) -> InterpretResult {
    let mut binding = self.main.borrow_mut();
    let module = binding.as_mut().unwrap();
    let mut sub_threads = vec![];
    for thread in &mut self.sub_threads {
      let data = thread.run_instruction();
      if matches!(data, InterpretResult::Continue) {
        sub_threads.push(thread.clone());
      }
    }
    self.sub_threads = sub_threads;
    module.run_instruction()
  }
  pub fn run(&mut self) -> InterpretResult {
    loop {
      let data = self.run_instruction();
      if matches!(data, InterpretResult::Ok) && self.sub_threads.is_empty() {
        return data;
      }
    }
  }
  pub fn interpret(&mut self) -> InterpretResult {
    let result = self.run();
    let thread = self.main.borrow().as_ref().unwrap().thread.clone();
    match &result {
      InterpretResult::RuntimeError(e) => thread.borrow_mut().runtime_error(&format!(
        "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
        e,
        call_stack_to_string(&thread.borrow_mut().call_stack)
      )),
      InterpretResult::CompileError(e) => {
        thread.borrow_mut().runtime_error(&format!("Error en compilacion\n\t{}", e))
      }
      _ => {}
    };
    let stack = thread.borrow_mut().stack.clone();
    if stack.len() != 0 {
      thread.borrow_mut().runtime_error(&format!("Error de pila no vacia | {stack:?}"));
    }
    result
  }
  pub fn resolve(this: Rc<RefCell<Option<Self>>>, path: &str, vars: Rc<RefCell<VarsManager>>) -> ModuleThread {
    let mut result = ModuleThread {
      module: Value::Object(Object::Map(HashMap::new().into(), HashMap::new().into())),
      path: path.to_string(),
      status: InterpretResult::Continue,
      sub_module: Rc::new(RefCell::new(None)),
      thread: Rc::new(RefCell::new(Thread::new(this.clone(), None)))
    };
    *result.thread.borrow().module.borrow_mut() = Some(result.clone());

    let file = match crate::code(path) {
      None => {
        result.status = InterpretResult::NativeError;
        return result;
      },
      Some(value) => value,
    };

    let ref ast = match crate::parser::Parser::new(file, &path).produce_ast() {
      Err(a) => {
        crate::parser::print_error(crate::parser::error_to_string(
          &crate::parser::ErrorNames::SyntaxError,
          crate::parser::node_error(&a),
        ));
        result.status = InterpretResult::NativeError;
        return result;
      }
      Ok(value) => value,
    };
    result.thread.borrow_mut().call_stack.push(CallFrame::new_compiler(ast.into(), vars));
    result
  }
}
