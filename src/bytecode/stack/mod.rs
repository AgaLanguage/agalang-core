use std::cell::RefCell;
use std::rc::Rc;

use super::compiler::{ChunkGroup, Compiler};
use super::value::Function;

mod vars_manager;
pub use vars_manager::VarsManager;

#[derive(PartialEq, Clone)]
pub enum InterpretResult {
  Ok,
  Continue,
  CompileError(String),
  RuntimeError(String),
  NativeError
}

#[derive(Clone)]
pub struct CallFrame {
  ip: usize,
  function: Function,
  locals: Vec<Rc<RefCell<VarsManager>>>,
}
impl CallFrame {
  pub fn new_compiler(compiler: Compiler, vars: Rc<RefCell<VarsManager>>) -> Self {
    Self::new(
      compiler.function,
      vec![Rc::new(RefCell::new(VarsManager::crate_child(vars)))],
    )
  }
  pub fn new(function: Function, locals: Vec<Rc<RefCell<VarsManager>>>) -> Self {
    Self {
      ip: 0,
      function,
      locals,
    }
  }
  pub fn current_chunk(&mut self) -> &mut ChunkGroup {
    self.function.chunk()
  }
  pub fn current_line(&mut self) -> usize {
    let instruction = self.ip.saturating_sub(1);
    let instruction = if instruction > self.ip {
      0
    } else {
      instruction
    };
    self.current_chunk().get_line(instruction)
  }
  pub fn read(&mut self) -> u8 {
    let ip = self.ip;
    let byte = self.current_chunk().read(ip);
    self.ip += 1;
    byte
  }
  pub fn back(&mut self, offset: usize) {
    self.ip -= offset;
  }
  pub fn advance(&mut self, offset: usize) {
    self.ip += offset;
  }
  pub fn current_vars(&self) -> Rc<RefCell<VarsManager>> {
    self.locals.last().unwrap().clone()
  }
  pub fn resolve_vars(&mut self, name: &str) -> Rc<RefCell<VarsManager>> {
    let mut vars = self.current_vars();
    for local in self.locals.clone() {
      if local.borrow().has(name) {
        vars = local
      }
    }
    vars
  }
  pub fn add_vars(&mut self) {
    self
      .locals
      .push(Rc::new(RefCell::new(VarsManager::crate_child(
        self.current_vars(),
      ))));
  }
  pub fn pop_vars(&mut self) -> Rc<RefCell<VarsManager>> {
    self.locals.pop().unwrap()
  }
}
impl std::fmt::Debug for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n\t{}", self.function.location())
  }
}
impl ToString for CallFrame {
  fn to_string(&self) -> String {
    self.function.to_string()
  }
}
pub fn call_stack_to_string(stack: &Vec<CallFrame>) -> String {
  let mut string = String::new();
  let mut index = stack.len();
  while index > 0 {
    index -= 1;
    string.push_str(&format!("\n\t{}", stack[index].function.location()));
  }
  string
}
