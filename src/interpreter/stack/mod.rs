use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::compiler::{Function, MultiRefHash};

mod vars_manager;
pub use vars_manager::VarsManager;

#[derive(PartialEq, Clone, Debug)]
pub enum InterpretResult {
  Ok,
  Continue,
  CompileError(String),
  RuntimeError(String),
}

#[derive(Clone)]
pub struct CallFrame {
  ip: usize,
  function: MultiRefHash<Function>,
  locals: Vec<Rc<RefCell<VarsManager>>>,
}
impl CallFrame {
  pub fn new_compiler(compiler: crate::compiler::Compiler, vars: Rc<RefCell<VarsManager>>) -> Self {
    Self::new(
      compiler.function.into(),
      vec![Rc::new(RefCell::new(VarsManager::crate_child(vars)))],
    )
  }
  pub fn new(function: MultiRefHash<Function>, locals: Vec<Rc<RefCell<VarsManager>>>) -> Self {
    Self {
      ip: 0,
      function,
      locals,
    }
  }
  pub fn current_chunk(&mut self) -> RefMut<crate::compiler::ChunkGroup> {
    RefMut::map(self.function.borrow_mut(), |func| func.chunk())
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
  pub fn peek(&mut self) -> u8 {
    let ip = self.ip;
    let byte = self.current_chunk().read(ip);
    byte
  }
  pub fn back(&mut self, offset: usize) {
    self.ip -= offset;
  }
  pub fn advance(&mut self, offset: usize) {
    self.ip += offset;
  }
  pub fn globals(&self) -> Rc<RefCell<VarsManager>> {
    self.locals[0].clone()
  }
  pub fn current_vars(&self) -> Rc<RefCell<VarsManager>> {
    self.locals.last().unwrap().clone()
  }
  pub fn resolve_vars(&mut self, name: &str) -> Rc<RefCell<VarsManager>> {
    let original = self.current_vars();
    let vars = Rc::new(RefCell::new(original.clone()));
    loop {
      let ref_vars = vars.borrow();
      let vars_manager = ref_vars.borrow();

      if vars_manager.has(name) {
        break;
      }

      let link = vars_manager.get_link();

      // no hacer provoca un error de prestamos
      drop(vars_manager);
      drop(ref_vars);

      if let Some(link) = link {
        *vars.borrow_mut() = link;
      } else {
        *vars.borrow_mut() = original;
        break;
      }
    }
    let x = vars.borrow().clone();
    x
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
  pub fn in_class(&self) -> Option<MultiRefHash<crate::compiler::Class>> {
    self.function.borrow().get_in_class()
  }
}
impl std::fmt::Debug for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n\t{}", self.function.borrow().location())
  }
}
impl ToString for CallFrame {
  fn to_string(&self) -> String {
    self.function.borrow().to_string()
  }
}
pub fn call_stack_to_string(stack: &Vec<CallFrame>) -> String {
  let mut string = String::new();
  let mut index = stack.len();
  while index > 0 {
    index -= 1;
    string.push_str(&format!(
      "\n\t{}",
      stack[index].function.borrow().location()
    ));
  }
  string
}
