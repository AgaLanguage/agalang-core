use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::env::VarError;
use std::fmt::format;
use std::rc::Rc;

use super::chunk::{Chunk, OpCode};
use super::value::{Value, FALSE_NAME, NEVER_NAME, NULL_NAME, TRUE_NAME};

const STACK_MAX: usize = 256;

type RC<T> = Rc<RefCell<T>>;
fn rc<T>(t: T) -> Rc<RefCell<T>> {
  Rc::new(RefCell::new(t))
}

#[derive(PartialEq)]
pub enum InterpretResult {
  Ok,
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

pub struct VM<'a> {
  chunk: Option<&'a Chunk>,
  ip: usize,
  stack: Vec<Value>,
  globals: RC<VarsManager>,
  locals: Vec<RC<VarsManager>>,
}

impl<'a> VM<'a> {
  pub fn new() -> Self {
    Self {
      chunk: None,
      ip: 0,
      stack: vec![],
      globals: rc(VarsManager::get_global()),
      locals: vec![],
    }
  }
  fn get_current_vars(&mut self) -> RC<VarsManager> {
    self.locals.last().unwrap_or_else(|| &self.globals).clone()
  }
  fn resolve(&mut self, name: &str) -> RC<VarsManager> {
    for local in &self.locals {
      if local.borrow().has(name) {
        return local.clone();
      }
    }
    self.globals.clone()
  }
  fn declare(&mut self, name: &str, value: Value, is_constant: bool) -> Option<Value> {
    self
      .get_current_vars()
      .borrow_mut()
      .declare(name, value, is_constant)
  }
  fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
    self.resolve(name).borrow_mut().assign(name, value)
  }
  fn get(&mut self, name: &str) -> Option<Value> {
    self.resolve(name).borrow().get(name).cloned()
  }
  pub fn free(&mut self) {
    self.chunk = None;
    self.ip = 0;
    self.reset_stack();
  }
  fn runtime_error(&mut self, message: &str) {
    eprintln!("{}", message);

    let chunk = self.chunk.unwrap();
    let instruction = self.ip.saturating_sub(1);
    if instruction < chunk.lines.len() {
      let line = chunk.lines[instruction];
      eprintln!("[linea {}] en script", line + 1);
    }

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
  fn peek(&self, distance: usize) -> Value {
    let index = self.stack.len() - 1 - distance;
    self.stack[index].clone()
  }
  pub fn interpret(&mut self, chunk: &'a Chunk) -> InterpretResult {
    self.chunk = Some(chunk);
    self.ip = 0;
    let result = self.run();
    match &result {
      InterpretResult::RuntimeError(e) => {
        self.runtime_error(&format!("Error en tiempo de ejecucion\n\t{}", e))
      }
      InterpretResult::CompileError(e) => self.runtime_error(&format!("Error en compilacion\n\t{}", e)),
      _ => {}
    };
    if self.stack.len() != 0 {
      self.runtime_error("Error de pila no vacia");
    }
    result
  }
  fn read_byte(&mut self) -> u8 {
    let byte = self.chunk.unwrap().read(self.ip);
    self.ip += 1;
    byte
  }
  fn read_constant(&mut self) -> Value {
    let constant_index = self.read_byte();
    self
      .chunk
      .unwrap()
      .constants
      .get(constant_index as usize)
      .clone()
  }
  fn read_string(&mut self) -> String {
    self.read_constant().asObject().asString()
  }
  fn read_short(&mut self) -> u16 {
    self.ip += 2;
    let a = self.chunk.unwrap().read(self.ip - 2) as u16;
    let b = self.chunk.unwrap().read(self.ip - 1) as u16;
    (a << 8) | b
  }
  fn run(&mut self) -> InterpretResult {
    let chunk = self.chunk.unwrap();
    loop {
      let byte_instruction = self.read_byte();
      let instruction: OpCode = byte_instruction.into();

      //println!("{:?} {:?}", instruction, self.stack);

      match instruction {
        OpCode::OpConstant | OpCode::OpConstantLong => {
          let constant = self.read_constant();
          self.push(constant);
        }
        OpCode::OpJumpIfFalse => {
          let jump = self.read_short() as usize;
          if self.peek(0).asBoolean() == false {
            self.ip += jump;
          }
        }
        OpCode::OpJump => {
          let jump = self.read_short() as usize;
          self.ip += jump;
        }
        OpCode::OpLoop => {
          let offset = self.read_short() as usize;
          self.ip -= offset;
        }
        OpCode::OpRemoveLocals => {
          self.locals.pop();
        }
        OpCode::OpNewLocals => {
          let current = self.get_current_vars();
          self.locals.push(rc(VarsManager::crate_child(current)))
        }
        OpCode::OpVarDecl => {
          let name = self.read_string();
          let value = self.peek(0);
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
          let value = self.peek(0);
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
          let value = self.peek(0);
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
          if a.isNumber() && b.isNumber() {
            let a = a.asNumber();
            let b = b.asNumber();
            self.push(Value::Number(a + b));
            continue;
          }
          let a_is_string = {
            if a.isObject() {
              a.asObject().isString()
            } else {
              false
            }
          };
          let b_is_string = {
            if b.isObject() {
              b.asObject().isString()
            } else {
              false
            }
          };
          if a_is_string || b_is_string {
            let a = a.asObject().asString();
            let b = b.asObject().asString();
            self.push(Value::Object(format!("{a}{b}").as_str().into()));
            continue;
          }
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a + b'"));
        }
        OpCode::OpSubtract => {
          let b = self.pop();
          let a = self.pop();
          if !a.isNumber() || !b.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar 'a - b'"));
          }
          let a = a.asNumber();
          let b = b.asNumber();
          self.push(Value::Number(a - b));
        }
        OpCode::OpMultiply => {
          let b = self.pop();
          let a = self.pop();
          if !a.isNumber() || !b.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar 'a * b'"));
          }
          let a = a.asNumber();
          let b = b.asNumber();
          self.push(Value::Number(a * b));
        }
        OpCode::OpDivide => {
          let b = self.pop();
          let a = self.pop();
          if !a.isNumber() || !b.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar 'a / b'"));
          }
          let a = a.asNumber();
          let b = b.asNumber();
          self.push(Value::Number(a / b));
        }
        OpCode::OpOr => {
          let b = self.pop();
          let a = self.pop();
          if a.asBoolean() {
            self.push(a);
          }
          self.push(b);
        }
        OpCode::OpAnd => {
          let b = self.pop();
          let a = self.pop();
          if !a.asBoolean() {
            self.push(a);
          }
          self.push(b);
        }
        OpCode::OpNegate => {
          let value = self.pop();
          if !value.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar '-x'"));
          }
          self.push(Value::Number(-value.asNumber()));
        }
        OpCode::OpNot => {
          let value = self.pop().asBoolean();
          let value = if value { Value::False } else { Value::True };
          self.push(value);
        }
        OpCode::OpAsBoolean => {
          let value = self.pop().asBoolean();
          let value = if value { Value::True } else { Value::False };
          self.push(value);
        }
        OpCode::OpAsString => {
          let value = self.pop().asObject().asString();
          let value = Value::Object(value.as_str().into());
          self.push(value);
        }
        OpCode::OpConsoleOut => {
          let value = self.pop().asObject().asString();
          print!("{value}");
          use std::io::Write as _;
          std::io::stdout().flush();
          self.push(Value::Never);
        }
        OpCode::OpReturn => {
          return InterpretResult::Ok;
        }

        OpCode::OpEquals => {
          let b = self.pop();
          let a = self.pop();
          let value = if a == b { Value::True } else { Value::False };
          self.push(value);
        }
        OpCode::OpGreaterThan => {
          let b = self.pop();
          let a = self.pop();
          if !a.isNumber() || !b.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar 'a > b'"));
          }
          let a = a.asNumber();
          let b = b.asNumber();
          let value = if a > b { Value::True } else { Value::False };
          self.push(value);
        }
        OpCode::OpLessThan => {
          let b = self.pop();
          let a = self.pop();
          if !a.isNumber() || !b.isNumber() {
            return InterpretResult::RuntimeError(format!("No se pudo operar 'a < b'"));
          }
          let a = a.asNumber();
          let b = b.asNumber();
          let value = if a < b { Value::True } else { Value::False };
          self.push(value);
        }

        OpCode::OpNull => return InterpretResult::CompileError(format!("Byte invalido {}",byte_instruction)),
      }
    }
  }
}
