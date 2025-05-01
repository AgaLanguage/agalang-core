use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::chunk::{ChunkGroup, OpCode};
use super::compiler::Compiler;
use super::value::{Function, Value, FALSE_NAME, NEVER_NAME, NULL_NAME, TRUE_NAME};

type RC<T> = Rc<RefCell<T>>;
fn rc<T>(t: T) -> Rc<RefCell<T>> {
  Rc::new(RefCell::new(t))
}

#[derive(PartialEq)]
pub enum InterpretResult {
  Ok,
  Continue,
  CompileError(String),
  RuntimeError(String),
}

struct VarsManager {
  variables: HashMap<String, Value>,
  constants: HashSet<String>,
  link: Option<RC<VarsManager>>,
}
impl VarsManager {
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      constants: HashSet::new(),
      link: None,
    }
  }
  pub fn get_global() -> Self {
    let mut this = Self::new();
    this.declare(NEVER_NAME, Value::Never, true);
    this.declare(NULL_NAME, Value::Null, true);
    this.declare(FALSE_NAME, Value::False, true);
    this.declare(TRUE_NAME, Value::True, true);
    this
  }
  pub fn crate_child(parent: RC<Self>) -> Self {
    let mut this = Self::new();
    this.link = Some(parent);
    this
  }
  pub fn declare(&mut self, name: &str, value: Value, is_constant: bool) -> Option<Value> {
    if self.variables.contains_key(name) {
      return None;
    }
    if is_constant {
      self.constants.insert(name.to_string());
    }
    self.variables.insert(name.to_string(), value.clone());
    Some(value)
  }
  pub fn has(&self, name: &str) -> bool {
    self.variables.contains_key(name)
  }
  pub fn get(&self, name: &str) -> Option<&Value> {
    self.variables.get(name)
  }
  pub fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
    if !self.variables.contains_key(name) || self.constants.contains(name) {
      return None;
    };
    self.variables.insert(name.to_string(), value.clone());
    Some(value)
  }
}

struct CallFrame {
  ip: usize,
  function: Function,
  locals: Vec<RC<VarsManager>>,
}
impl CallFrame {
  pub fn new(compiler: Compiler, vars: RC<VarsManager>) -> Self {
    Self {
      ip: 0,
      function: compiler.function,
      locals: vec![rc(VarsManager::crate_child(vars))],
    }
  }
  fn current_chunk(&mut self) -> &mut ChunkGroup {
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
  pub fn current_vars(&self) -> RC<VarsManager> {
    self.locals.last().unwrap().clone()
  }
  pub fn resolve_vars(&mut self, name: &str) -> RC<VarsManager> {
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
      .push(rc(VarsManager::crate_child(self.current_vars())));
  }
  pub fn pop_vars(&mut self) -> RC<VarsManager> {
    self.locals.pop().unwrap()
  }
}
impl std::fmt::Debug for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n\t{}", self.function.location())
  }
}
fn call_stack_to_string(stack: &Vec<CallFrame>) -> String {
  let mut string = String::new();
  let mut index = stack.len();
  while index > 0 {
    index -= 1;
    string.push_str(&format!("\n\t{}", stack[index].function.location()));
  }
  string
}
pub struct VM {
  stack: Vec<Value>,
  globals: RC<VarsManager>,
  call_stack: Vec<CallFrame>,
}

impl VM {
  pub fn new(compiler: Compiler) -> Self {
    //compiler.chunk().print();
    let globals = rc(VarsManager::get_global());
    Self {
      stack: vec![],
      call_stack: vec![CallFrame::new(compiler, globals.clone())],
      globals,
    }
  }
  fn current_frame(&mut self) -> &mut CallFrame {
    self.call_stack.last_mut().unwrap()
  }
  fn current_chunk(&mut self) -> &mut ChunkGroup {
    self.current_frame().function.chunk()
  }
  fn current_vars(&mut self) -> RC<VarsManager> {
    self.current_frame().current_vars()
  }
  fn resolve(&mut self, name: &str) -> RC<VarsManager> {
    let mut vars = self.globals.clone();
    for call in &mut self.call_stack {
      let local_vars = call.resolve_vars(name);
      if local_vars.borrow().has(name) {
        vars = local_vars;
      }
    }
    vars
  }
  fn declare(&mut self, name: &str, value: Value, is_constant: bool) -> Option<Value> {
    self
      .current_vars()
      .borrow_mut()
      .declare(name, value, is_constant)
  }
  fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
    self.resolve(name).borrow_mut().assign(name, value)
  }
  fn get(&mut self, name: &str) -> Option<Value> {
    self.resolve(name).borrow().get(name).cloned()
  }
  fn runtime_error(&mut self, message: &str) {
    eprintln!("{}", message);
    eprintln!(
      "[linea {}] en {}",
      self.current_frame().current_line(),
      self.current_frame().function.to_string()
    );

    self.reset_stack();
  }
  fn reset_stack(&mut self) {
    self.stack = vec![];
  }
  pub fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  pub fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }
  pub fn interpret(&mut self) -> InterpretResult {
    let result = self.run();
    match &result {
      InterpretResult::RuntimeError(e) => self.runtime_error(&format!(
        "Error en tiempo de ejecucion\n\t{}\n\t{}\n",
        e,
        call_stack_to_string(&self.call_stack)
      )),
      InterpretResult::CompileError(e) => {
        self.runtime_error(&format!("Error en compilacion\n\t{}", e))
      }
      _ => {}
    };
    if self.stack.len() != 0 {
      self.runtime_error(&format!("Error de pila no vacia | {:?}", self.stack));
    }
    result
  }
  fn read(&mut self) -> u8 {
    self.current_frame().read()
  }
  fn read_constant(&mut self) -> Value {
    let constant_index = self.read();
    self.current_chunk().read_constant(constant_index).clone()
  }
  fn read_string(&mut self) -> String {
    self.read_constant().as_string()
  }
  fn read_short(&mut self) -> u16 {
    let a = self.read() as u16;
    let b = self.read() as u16;
    (a << 8) | b
  }
  fn call_value(&mut self, callee: Value, arity: usize) -> bool {
    if !callee.is_object() {
      if !callee.is_number() {
        return false;
      }
      if arity != 1 {
        return false;
      }
      let _num = callee.as_number();
    }
    let obj = callee.as_object();
    if !obj.is_function() {
      return false;
    }
    let function = obj.as_function();
    self.call_stack.push(CallFrame {
      ip: 0,
      function,
      locals: vec![rc(VarsManager::crate_child(self.globals.clone()))],
    });
    true
  }
  pub fn run_instruction(&mut self) -> InterpretResult {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    println!("{:<16} | {:?}", format!("{:?}", instruction), self.stack);

    match instruction {
      OpCode::OpApproximate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar '~x'"));
        }
        self.push(Value::Number(value.as_number().trunc()));
      }
      OpCode::OpSetMember => {
        let value = self.pop();
        let key = self.pop();
        let object = self.pop();
        if !object.is_object() {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para obtener la propiedad"
          ));
        }
        let obj = object.as_object();
        if obj.is_map() {
          let _is_instance = self.read() == 1u8;
          let map = obj.as_map();
          let value = map
            .borrow_mut()
            .insert(key.as_string(), value)
            .unwrap_or_default();
          self.stack.push(value);
        } else if obj.is_array() {
          let _is_instance = self.read() == 1u8;
          let vec = obj.as_array();
          if !key.is_number() {
            return InterpretResult::RuntimeError(format!("Se esperaba un indice de propiedad"));
          }
          let key = key.as_number();
          let index = key.abs().trunc();
          if key != index {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser un numero entero positivo"
            ));
          }
          let index = index.to_string().parse::<usize>().unwrap_or(0);
          if vec.borrow().len() <= index {
            vec.borrow_mut().push(value.clone());
          } else {
            vec.borrow_mut()[index] = value.clone();
          };
          self.stack.push(value);
        } else {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para obtener la propiedad"
          ));
        }
      }
      OpCode::OpGetMember => {
        let key = self.pop();
        let object = self.pop();
        let is_instance = self.read() == 1u8;
        if is_instance {
          let key: &str = &key.as_string();
          if let Some(value) = object.get_instance_property(key) {
            self.push(value);
            return InterpretResult::Continue;
          }
          let type_name = object.get_type();
          return InterpretResult::RuntimeError(format!(
            "No se pudo obtener la propiedad de instancia '{key}' de '{type_name}'"
          ));
        }
        if !object.is_object() {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para obtener la propiedad"
          ));
        }
        let obj = object.as_object();
        if obj.is_map() {
          let map = obj.as_map();
          let value = map
            .borrow()
            .get(&key.as_string())
            .cloned()
            .unwrap_or_default();
          self.stack.push(value);
        } else if obj.is_array() {
          let vec = obj.as_array();
          if !key.is_number() {
            return InterpretResult::RuntimeError(format!("Se esperaba un indice de propiedad"));
          }
          let key = key.as_number();
          let index = key.abs().trunc();
          if key != index {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser un numero entero positivo"
            ));
          }
          let index = index.to_string().parse::<usize>().unwrap_or(0);
          let value = vec.borrow().get(index).cloned().unwrap_or_default();
          self.stack.push(value);
        } else {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para obtener la propiedad"
          ));
        }
      }
      OpCode::OpConstant => {
        let constant = self.read_constant();
        self.push(constant);
      }
      OpCode::OpJumpIfFalse => {
        let jump = self.read_short() as usize;
        if self.pop().as_boolean() == false {
          self.current_frame().advance(jump);
        }
      }
      OpCode::OpArgDecl => {
        let name = self.read_constant().as_string();
        let value = self.pop();
        match self.declare(&name, value, true) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo declarar la variable '{name}'"
            ))
          }
          _ => {}
        }
      }
      OpCode::OpJump => {
        let jump = self.read_short() as usize;
        self.current_frame().advance(jump);
      }
      OpCode::OpLoop => {
        let offset = self.read_short() as usize;
        self.current_frame().back(offset);
      }
      OpCode::OpRemoveLocals => {
        self.current_frame().pop_vars();
      }
      OpCode::OpNewLocals => {
        self.current_vars();
        self.current_frame().add_vars()
      }
      OpCode::OpCall => {
        let arity = self.read() as usize;
        let callee = self.pop();
        let call = self.call_value(callee, arity);
        if !call {
          return InterpretResult::RuntimeError("Se esperaba llamar a una funcion".into());
        }
      }
      OpCode::OpVarDecl => {
        let name = self.read_string();
        let value = self.pop();
        match self.declare(&name, value, false) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo declarar la variable '{name}'"
            ))
          }
          _ => {}
        }
      }
      OpCode::OpConstDecl => {
        let name = self.read_string();
        let value = self.pop();
        match self.declare(&name, value, true) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo declarar la constante '{name}'"
            ))
          }
          _ => {}
        }
      }
      OpCode::OpGetVar => {
        let name = self.read_string();
        let value = {
          let v = self.get(&name);
          match v {
            None => {
              return InterpretResult::RuntimeError(format!(
                "No se pudo obtener la variable '{name}'"
              ))
            }
            Some(value) => value.clone(),
          }
        };
        self.push(value);
      }
      OpCode::OpSetVar => {
        let name = self.read_string();
        let value = self.pop();
        match self.assign(&name, value) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo re-asignar la variable '{name}'"
            ))
          }
          _ => {}
        };
      }
      OpCode::OpPop => {
        self.pop();
      }
      OpCode::OpAdd => {
        let b = self.pop();
        let a = self.pop();
        if a.is_number() && b.is_number() {
          let a = a.as_number();
          let b = b.as_number();
          self.push(Value::Number(a + b));
          return InterpretResult::Continue;
        }
        if a.is_string() || b.is_string() {
          let a = a.as_string();
          let b = b.as_string();
          self.push(Value::Object(format!("{a}{b}").as_str().into()));
          return InterpretResult::Continue;
        }
        return InterpretResult::RuntimeError(format!("No se pudo operar 'a + b'"));
      }
      OpCode::OpSubtract => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a - b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        self.push(Value::Number(a - b));
      }
      OpCode::OpMultiply => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a * b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        self.push(Value::Number(a * b));
      }
      OpCode::OpDivide => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a / b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        self.push(Value::Number(a / b));
      }
      OpCode::OpOr => {
        let b = self.pop();
        let a = self.pop();
        if a.as_boolean() {
          self.push(a);
        }
        self.push(b);
      }
      OpCode::OpAnd => {
        let b = self.pop();
        let a = self.pop();
        if !a.as_boolean() {
          self.push(a);
        }
        self.push(b);
      }
      OpCode::OpNegate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar '-x'"));
        }
        self.push(Value::Number(-value.as_number()));
      }
      OpCode::OpNot => {
        let value = self.pop().as_boolean();
        let value = if value { Value::False } else { Value::True };
        self.push(value);
      }
      OpCode::OpAsBoolean => {
        let value = self.pop().as_boolean();
        let value = if value { Value::True } else { Value::False };
        self.push(value);
      }
      OpCode::OpAsString => {
        let value = self.pop().as_string();
        let value = Value::Object(value.as_str().into());
        self.push(value);
      }
      OpCode::OpConsoleOut => {
        let value = self.pop().as_string();
        print!("{value}");
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        self.push(Value::Never);
      }
      OpCode::OpReturn => {
        self.call_stack.pop();
        let value = self.pop();
        if self.call_stack.len() == 0 {
          return InterpretResult::Ok;
        }
        self.push(value);
      }

      OpCode::OpEquals => {
        let b = self.pop();
        let a = self.pop();
        if a.is_number() && b.is_number() {
          let a = a.as_number();
          let b = b.as_number();
          let value = if a.is_nan() || b.is_nan() {
            Value::False
          } else if a == b {
            Value::True
          } else {
            Value::False
          };
          self.push(value);
          return InterpretResult::Continue;
        }
        let value = if a == b { Value::True } else { Value::False };
        self.push(value);
      }
      OpCode::OpGreaterThan => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a > b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        let value = if a > b { Value::True } else { Value::False };
        self.push(value);
      }
      OpCode::OpLessThan => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} < {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number();
        let b = b.as_number();
        let value = if a < b { Value::True } else { Value::False };
        self.push(value);
      }

      OpCode::OpBreak | OpCode::OpContinue => {
        self.push(Value::Null);
      }
      OpCode::OpNull => {
        return InterpretResult::CompileError(format!("Byte invalido {}", byte_instruction))
      }
    }
    return InterpretResult::Continue;
  }

  fn run(&mut self) -> InterpretResult {
    loop {
      let result = self.run_instruction();
      match result {
        InterpretResult::Continue => {}
        _ => return result,
      }
    }
  }
}
