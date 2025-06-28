use std::fmt::Display;

use crate::{compiler::Function, MultiRefHash};

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
  locals: Vec<MultiRefHash<VarsManager>>,
}
impl CallFrame {
  pub fn new_compiler(
    compiler: crate::compiler::Compiler,
    vars: MultiRefHash<VarsManager>,
  ) -> Self {
    Self::new(
      compiler.function.into(),
      vec![VarsManager::crate_child(vars).into()],
    )
  }
  pub fn new(function: MultiRefHash<Function>, locals: Vec<MultiRefHash<VarsManager>>) -> Self {
    Self {
      ip: 0,
      function,
      locals,
    }
  }
  pub fn current_chunk(&self) -> MultiRefHash<crate::compiler::ChunkGroup> {
    self.function.write().chunk()
  }
  pub fn current_line(&self) -> usize {
    let instruction = self.ip.saturating_sub(1);
    let instruction = if instruction > self.ip {
      0
    } else {
      instruction
    };
    self.current_chunk().read().get_line(instruction)
  }
  pub fn read(&mut self) -> u8 {
    let byte = self.current_chunk().read().read(self.ip);
    self.ip += 1;
    byte
  }
  pub fn peek(&self) -> u8 {
    self.current_chunk().read().read(self.ip)
  }
  pub fn back(&mut self, offset: usize) {
    self.ip -= offset;
  }
  pub fn advance(&mut self, offset: usize) {
    self.ip += offset;
  }
  pub fn globals(&self) -> MultiRefHash<VarsManager> {
    self.locals.first().unwrap().clone()
  }
  pub fn current_vars(&self) -> MultiRefHash<VarsManager> {
    self.locals.last().unwrap().clone()
  }
  pub fn resolve_vars(&self, name: &str) -> MultiRefHash<VarsManager> {
    let original = self.current_vars();
    let vars: MultiRefHash<MultiRefHash<VarsManager>> = original.clone().into();
    loop {
      let ref_vars = vars.read();
      let vars_manager = ref_vars.read();

      if vars_manager.has(name) {
        break;
      }

      let link = vars_manager.get_link();

      // no hacer provoca un error de prestamos
      drop(vars_manager);
      drop(ref_vars);

      if let Some(link) = link {
        *vars.write() = link;
      } else {
        *vars.write() = original;
        break;
      }
    }
    let x = vars.read().clone();
    x
  }
  pub fn add_vars(&mut self) {
    self
      .locals
      .push(VarsManager::crate_child(self.current_vars()).into());
  }
  pub fn pop_vars(&mut self) -> MultiRefHash<VarsManager> {
    self.locals.pop().unwrap()
  }
  pub fn in_class(&self) -> Option<MultiRefHash<crate::compiler::Class>> {
    self.function.read().get_in_class()
  }
}
impl std::fmt::Debug for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n\t{}", self.function.read().location())
  }
}
impl Display for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.function.read())
  }
}
pub fn call_stack_to_string(stack: &[CallFrame]) -> String {
  let mut string = String::new();
  let mut index = stack.len();
  while index > 0 {
    index -= 1;
    string.push_str(&format!("\n\t{}", stack[index].function.read().location()));
  }
  string
}
