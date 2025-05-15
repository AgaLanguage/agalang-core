use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use super::cache::Cache;
use super::compiler::{ChunkGroup, Compiler, OpCode};
use super::libs::libs;
use super::stack::{call_stack_to_string, CallFrame, InterpretResult, VarsManager};
use super::value::{Function, Value, Object};

pub struct VM {
  path: Box<Path>,
  module: Value,
  stack: Vec<Value>,
  globals: Rc<RefCell<VarsManager>>,
  call_stack: Vec<CallFrame>,
  cache: Cache,
}

impl VM {
  pub fn new(compiler: Compiler) -> Self {
    let compiler = {
      let mut compiler = compiler;
      compiler.function.chunk().print();
      compiler
    };
    let globals = Rc::new(RefCell::new(VarsManager::get_global()));
    Self {
      path: Path::new(&compiler.path).into(),
      module: Value::Object(Object::Map(
        HashMap::new().into(),
        HashMap::new().into(),
      )),
      stack: vec![],
      call_stack: vec![CallFrame::new_compiler(compiler, globals.clone())],
      globals,
      cache: Cache::new(),
    }
  }
  pub fn as_value(self) -> Value {
    self.module
  }
  fn current_frame(&mut self) -> &mut CallFrame {
    self.call_stack.last_mut().unwrap()
  }
  fn current_chunk(&mut self) -> &mut ChunkGroup {
    self.current_frame().current_chunk()
  }
  fn current_vars(&mut self) -> Rc<RefCell<VarsManager>> {
    self.current_frame().current_vars()
  }
  fn resolve(&mut self, name: &str) -> Rc<RefCell<VarsManager>> {
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
      self.current_frame().to_string()
    );

    self.reset_stack();
  }
  fn reset_stack(&mut self) {
    self.stack = vec![];
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Value {
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
  fn call_value(&mut self, this: Value, callee: Value, arity: usize) -> InterpretResult {
    let mut args = vec![];
    for _ in 0..arity {
      args.push(self.pop());
    }
    args.reverse();
    if !callee.is_object() {
      if !callee.is_number() {
        return InterpretResult::RuntimeError("Se esperaba llamar una funcion [1]".into());
      }
      if arity != 1 || args.len() != 1 {
        return InterpretResult::RuntimeError(
          "Solo se puede multiplicar un numero (llamada)".into(),
        );
      }
      let arg = args.get(0).unwrap();
      if !arg.is_number() {
        return InterpretResult::RuntimeError(
          "Solo se pueden multiplicar numeros (llamada)".into(),
        );
      }
      let num = callee.as_number();
      let arg = arg.as_number();
      let value = Value::Number(num * arg);
      self.push(value);
      return InterpretResult::Continue;
    }
    let obj = callee.as_object();
    if !obj.is_function() {
      return InterpretResult::RuntimeError("Se esperaba llamar una funcion [2]".into());
    }
    let function = obj.as_function();
    if let Function::Native { func, .. } = function {
      let value = func(this, args);
      return if let Err(error) = value {
        InterpretResult::RuntimeError(error)
      } else {
        self.push(value.unwrap());
        InterpretResult::Continue
      };
    }
    let locals = vec![Rc::new(RefCell::new(
      VarsManager::crate_child(self.globals.clone()).set_this(this),
    ))];
    self.call_stack.push(CallFrame::new(function, locals));
    InterpretResult::Continue
  }
  fn run_instruction(&mut self) -> InterpretResult {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    // println!("{:<16} | {:?}", format!("{:?}", instruction), self.stack);

    match instruction {
      OpCode::OpCopy => {
        let value = self.pop();
        self.push(value.clone());
        self.push(value);
      }
      OpCode::OpApproximate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar '~x'"));
        }
        self.push(Value::Number(value.as_number().round()));
      }
      OpCode::OpSetMember => {
        let value = self.pop();
        let key = self.pop();
        let object = self.pop();
        let is_instance = self.read() == 1u8;
        if !object.is_object() {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para asignar la propiedad '{}' [1]",
            key.as_string()
          ));
        }
        if is_instance {
          return InterpretResult::RuntimeError(format!(
            "Las propiedades de instancia no se pueden asignar fuera de su clase"
          ));
        }
        let obj = object.as_object();
        if let Value::Never = value {
          let type_name = object.get_type();
          return InterpretResult::RuntimeError(format!(
            "No se puede asignar un valor '{type_name}' a una propiedad en su lugar usa '{}'",
            Value::Null.get_type()
          ));
        }
        if obj.is_map() {
          let map = obj.as_map();
          let mut map = map.0.borrow_mut();
          let value = map.insert(key.as_string(), value).unwrap_or_default();
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
          let mut vec = vec.borrow_mut();
          if index >= vec.len() {
            vec.resize(index + 1, Value::Never);
          }
          vec[index] = value.clone();
          self.stack.push(value);
        } else {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para asignar la propiedad '{}' [2]",
            key.as_string()
          ));
        }
      }
      OpCode::OpGetMember => {
        let key = self.pop();
        let object = self.pop();
        let is_instance = self.read() == 1u8;
        if is_instance {
          let key: &str = &key.as_string();
          if let Some(value) = object.get_instance_property(key, self.cache.proto.clone()) {
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
            "Se esperaba un objeto para obtener la propiedad '{}' [3]",
            key.as_string()
          ));
        }
        let obj = object.as_object();
        if obj.is_map() {
          let map = obj.as_map();
          let value = if is_instance {
            map.0.borrow().get(&key.as_string()).cloned()
          } else {
            map.1.borrow().get(&key.as_string()).cloned()
          };
          let value = if let Some(value) = value {
            value
          } else if is_instance {
            return InterpretResult::RuntimeError(format!(
              "No se pudo obtener la propiedad de instancia '{}' de '{}'",
              key.as_string(),
              object.get_type()
            ));
          } else {
            Value::Never
          };
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
            "Se esperaba un objeto para obtener la propiedad '{}' [4]",
            key.as_string()
          ));
        }
      }
      OpCode::OpConstant => {
        let constant = self.read_constant();
        self.push(constant);
      }
      OpCode::OpImport => {
        let module = self.pop();
        let path = module.as_string();
        let meta_byte = self.read();
        let name_byte = self.read();
        let _is_lazy = (meta_byte & 0b10) == 0b10;
        let alias = (meta_byte & 0b01) == 0b01;

        let lib_name = if path.starts_with(":") {
          path
        } else {
          self
            .path
            .parent()
            .unwrap()
            .join(path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
        };
        let value = libs(lib_name, self.cache.libs.clone());
        if alias {
          let name = self.current_chunk().read_constant(name_byte).as_string();
          self.declare(&name, value, true);
        }
        self.push(module);
      }
      OpCode::OpJumpIfFalse => {
        let jump = self.read_short() as usize;
        if self.pop().as_boolean() == false {
          self.current_frame().advance(jump);
        }
      }
      OpCode::OpArgDecl => {
        let name = self.read_string();
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
        let this = self.pop();
        return self.call_value(this, callee, arity);
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
