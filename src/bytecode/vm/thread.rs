use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::bytecode::compiler::{ChunkGroup, OpCode};
use crate::bytecode::libs::libs;
use crate::bytecode::stack::{CallFrame, InterpretResult, VarsManager};
use crate::bytecode::value::{Function, Value};

use super::{ModuleThread, VM};

#[derive(Clone)]
pub struct Thread {
  pub stack: Vec<Value>,
  pub call_stack: Vec<CallFrame>,
  pub module: Rc<RefCell<Option<ModuleThread>>>,
  pub vm: Rc<RefCell<Option<VM>>>,
}

impl Thread {
  pub fn new(
    vm: Rc<RefCell<Option<VM>>>,
    module: Option<Rc<RefCell<Option<ModuleThread>>>>,
  ) -> Self {
    let module = if let Some(module) = module {
      module
    } else {
      Rc::new(RefCell::new(None))
    };
    Self {
      stack: vec![],
      call_stack: vec![],
      module,
      vm,
    }
  }
  pub fn current_frame(&mut self) -> &mut CallFrame {
    self.call_stack.last_mut().unwrap()
  }
  fn current_chunk(&mut self) -> &mut ChunkGroup {
    self.current_frame().current_chunk()
  }
  fn current_vars(&mut self) -> Rc<RefCell<VarsManager>> {
    self.current_frame().current_vars()
  }
  fn resolve(&mut self, name: &str) -> Rc<RefCell<VarsManager>> {
    let mut vars = self.vm.borrow().as_ref().unwrap().globals.clone();
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
  pub fn runtime_error(&mut self, message: &str) {
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
    for arg in args {
      self.push(arg);
    }
    let vars = self.current_vars();
    let locals = vec![Rc::new(RefCell::new(
      VarsManager::crate_child(vars.clone()).set_this(this),
    ))];
    self.call_stack.push(CallFrame::new(function, locals));
    InterpretResult::Continue
  }
  pub fn run_instruction(&mut self) -> InterpretResult {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    //println!("{:<16} | {:?}", format!("{:?}", instruction), self.stack);

    let value: Value = match instruction {
      OpCode::OpCopy => {
        let value = self.pop();
        self.push(value.clone());
        value
      }
      OpCode::OpApproximate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar '~x'"));
        }
        Value::Number(value.as_number().round())
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
          value
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
          value
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
          let key = key.as_string();
          let proto = self.vm.borrow().as_ref().unwrap().cache.proto.clone();
          if let Some(value) = object.get_instance_property(&key, proto) {
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
          let value = map.0.borrow().get(&key.as_string()).cloned();
          let value = if let Some(value) = value {
            value
          } else {
            Value::Never
          };
          value
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
          value
        } else {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para obtener la propiedad '{}' [4]",
            key.as_string()
          ));
        }
      }
      OpCode::OpConstant => self.read_constant(),
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
          Path::new(&self.module.borrow().as_ref().unwrap().path)
            .parent()
            .unwrap()
            .join(path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
        };
        let proto = self.vm.borrow().as_ref().unwrap().cache.libs.clone();
        let value = libs(lib_name, proto, |path| {
          let thread = Rc::new(RefCell::new(VM::resolve(
            self.vm.clone(),
            path,
            self.current_vars(),
          )));
          *self
            .module
            .borrow()
            .as_ref()
            .unwrap()
            .sub_module
            .borrow_mut() = Some(thread.clone());
          let x = thread.borrow().clone().as_value();
          x
        });
        if alias {
          let name = self.current_chunk().read_constant(name_byte).as_string();
          self.declare(&name, value, true);
        }
        module
      }
      OpCode::OpJumpIfFalse => {
        let jump = self.read_short() as usize;
        if self.pop().as_boolean() == false {
          self.current_frame().advance(jump);
        }
        return InterpretResult::Continue;
      }
      OpCode::OpArgDecl => {
        let name = self.read_string();
        let value = self.pop();
        return match self.declare(&name, value.clone(), true) {
          None => {
            InterpretResult::RuntimeError(format!("No se pudo declarar la variable '{name}'"))
          }
          _ => InterpretResult::Continue,
        };
      }
      OpCode::OpJump => {
        let jump = self.read_short() as usize;
        self.current_frame().advance(jump);
        return InterpretResult::Continue;
      }
      OpCode::OpLoop => {
        let offset = self.read_short() as usize;
        self.current_frame().back(offset);
        return InterpretResult::Continue;
      }
      OpCode::OpRemoveLocals => {
        self.current_frame().pop_vars();
        return InterpretResult::Continue;
      }
      OpCode::OpNewLocals => {
        self.current_vars();
        self.current_frame().add_vars();
        return InterpretResult::Continue;
      }
      OpCode::OpCall => {
        let arity = self.read() as usize;
        let callee = self.pop();
        let this = self.pop();
        return self.call_value(this, callee, arity);
      }
      OpCode::OpExport => {
        let name = self.pop().as_string();
        let value = if let Some(value) = self.get(&name) {
          value.clone()
        } else {
          return InterpretResult::RuntimeError(format!(
            "Se exporto la funcion '{name}' antes de ser declarada"
          ));
        };
        let module = self.module.borrow().as_ref().unwrap().module.clone();
        if !module.is_object() {
          return InterpretResult::RuntimeError("Se esperaba un objeto como modulo".to_string());
        }
        let obj = module.as_object();
        if !obj.is_map() {
          return InterpretResult::RuntimeError("Se esperaba un objeto como modulo".to_string());
        }
        obj.as_map().1.borrow_mut().insert(name, value.clone());
        value
      }
      OpCode::OpVarDecl => {
        let name = self.read_string();
        let value = self.pop();
        return match self.declare(&name, value.clone(), false) {
          None => {
            InterpretResult::RuntimeError(format!("No se pudo declarar la variable '{name}'"))
          }
          _ => InterpretResult::Continue,
        };
      }
      OpCode::OpConstDecl => {
        let name = self.read_string();
        let value = self.pop();
        self.push(value.clone());
        return match self.declare(&name, value.clone(), true) {
          None => {
            InterpretResult::RuntimeError(format!("No se pudo declarar la constante '{name}'"))
          }
          _ => InterpretResult::Continue,
        };
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
        value
      }
      OpCode::OpSetVar => {
        let name = self.read_string();
        let value = self.pop();
        match self.assign(&name, value.clone()) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo re-asignar la variable '{name}'"
            ))
          }
          _ => value,
        }
      }
      OpCode::OpPop => {
        self.pop();
        return InterpretResult::Continue;
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
        Value::Number(a - b)
      }
      OpCode::OpMultiply => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a * b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a * b)
      }
      OpCode::OpDivide => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a / b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a / b)
      }
      OpCode::OpOr => {
        let b = self.pop();
        let a = self.pop();
        if a.as_boolean() {
          a
        } else {
          b
        }
      }
      OpCode::OpAnd => {
        let b = self.pop();
        let a = self.pop();
        if !a.as_boolean() {
          a
        } else {
          b
        }
      }
      OpCode::OpNegate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar '-x'"));
        }
        Value::Number(-value.as_number())
      }
      OpCode::OpNot => {
        let value = self.pop().as_boolean();
        let value = if value { Value::False } else { Value::True };
        value
      }
      OpCode::OpAsBoolean => {
        let value = self.pop().as_boolean();
        let value = if value { Value::True } else { Value::False };
        value
      }
      OpCode::OpAsString => {
        let value = self.pop().as_string();
        let value = Value::Object(value.as_str().into());
        value
      }
      OpCode::OpConsoleOut => {
        let value = self.pop().as_string();
        print!("{value}");
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        Value::Never
      }
      OpCode::OpReturn => {
        self.call_stack.pop();
        let value = self.pop();
        if self.call_stack.len() == 0 {
          return InterpretResult::Ok;
        }
        value
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
        if a == b {
          Value::True
        } else {
          Value::False
        }
      }
      OpCode::OpGreaterThan => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!("No se pudo operar 'a > b'"));
        }
        let a = a.as_number();
        let b = b.as_number();
        if a > b {
          Value::True
        } else {
          Value::False
        }
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
        value
      }
      OpCode::OpBreak | OpCode::OpContinue => Value::Null,
      OpCode::OpNull => {
        return InterpretResult::CompileError(format!("Byte invalido {}", byte_instruction))
      }
    };
    self.push(value);
    return InterpretResult::Continue;
  }
}
