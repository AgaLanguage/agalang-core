use std::cell::{RefCell, RefMut};
use std::path::Path;
use std::rc::Rc;

use crate::bytecode::compiler::{ChunkGroup, OpCode};
use crate::bytecode::libs::libs;
use crate::bytecode::stack::{CallFrame, InterpretResult, VarsManager};
use crate::bytecode::value::{Function, Value, REF_TYPE};
use crate::bytecode::vm::{AsyncThread, BlockingThread};

use super::{ModuleThread, VM};

#[derive(Clone, Debug)]
pub struct Thread {
  stack: Vec<Value>,
  call_stack: Vec<CallFrame>,
  module: Rc<RefCell<Option<ModuleThread>>>,
  vm: Rc<RefCell<VM>>,
}

impl Thread {
  pub fn new(vm: Rc<RefCell<VM>>) -> Rc<RefCell<Self>> {
    let module = vm.borrow().module.clone();
    Rc::new(RefCell::new(Self {
      stack: vec![],
      call_stack: vec![],
      module,
      vm,
    }))
  }
  pub fn set_module(&self, module: ModuleThread) {
    *self.module.borrow_mut() = Some(module);
  }
  pub fn get_stack(&self) -> &Vec<Value> {
    &self.stack
  }
  pub fn get_calls(&self) -> &Vec<CallFrame> {
    &self.call_stack
  }
  pub fn get_vm(&self) -> Rc<RefCell<VM>> {
    self.vm.clone()
  }
  pub fn clear_stack(&mut self) {
    self.stack.clear();
  }
  pub fn push_call(&mut self, frame: CallFrame) {
    self.call_stack.push(frame);
  }
  pub fn current_frame(&mut self) -> &mut CallFrame {
    self.call_stack.last_mut().unwrap()
  }
  fn current_chunk(&mut self) -> RefMut<ChunkGroup> {
    self.current_frame().current_chunk()
  }
  fn globals(&mut self) -> Rc<RefCell<VarsManager>> {
    self.current_frame().globals()
  }
  fn current_vars(&mut self) -> Rc<RefCell<VarsManager>> {
    self.current_frame().current_vars()
  }
  fn resolve(&mut self, name: &str) -> Rc<RefCell<VarsManager>> {
    let mut vars = self.vm.borrow().globals.clone();
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
    let frame = self.call_stack.last_mut();
    let (line, name) = match frame {
      Some(frame) => (frame.current_line().to_string(), frame.to_string()),
      None => ("?".to_string(), "?".to_string()),
    };
    eprintln!("[linea {line}] en {name}\n{message}");

    self.reset_stack();
  }
  fn reset_stack(&mut self) {
    self.stack = vec![];
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  pub fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }
  fn read(&mut self) -> u8 {
    self.current_frame().read()
  }
  pub fn peek(&mut self) -> OpCode {
    self.current_frame().peek().into()
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
      let value = self.pop();
      if value.is_iterator() {
        let mut list = match value.as_strict_array() {
          Ok(list) => list,
          Err(error) => {
            return InterpretResult::RuntimeError(error);
          }
        };
        list.reverse();
        for item in list.iter() {
          args.push(item.clone());
        }
      } else {
        args.push(value);
      }
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
    if !callee.is_function() {
      return InterpretResult::RuntimeError("Se esperaba llamar una funcion [2]".into());
    }
    let fun = callee.as_function();
    let fun_clone = fun.clone();
    let function = fun_clone.borrow();

    let (arity, has_rest, is_async) = match &*function {
      Function::Function {
        arity,
        has_rest,
        is_async,
        ..
      } => (*arity, *has_rest, *is_async),
      Function::Script { .. } => (0, false, false),
      Function::Native { func, .. } => {
        let value = func(this, args, self.get_vm().borrow().sub_threads.clone());
        return if let Err(error) = value {
          InterpretResult::RuntimeError(error)
        } else {
          self.push(value.unwrap());
          InterpretResult::Continue
        };
      }
    };
    // En el caso de que la funcion no tenga un scope definido, se usa el scope actual (esto deberia de pasar)
    let vars = function.get_scope().unwrap_or_else(|| self.current_vars());
    let locals = vec![Rc::new(RefCell::new(
      VarsManager::crate_child(vars.clone()).set_this(this.clone()),
    ))];
    if is_async {}
    self.call_stack.push(CallFrame::new(fun, locals));

    if arity > args.len() {
      if arity == 1 && args.len() == 0 {
        return InterpretResult::RuntimeError(
          "Se esperaba llamar una funcion con un argumento".into(),
        );
      }
      return InterpretResult::RuntimeError(format!(
        "Se esperaban {} argumentos, pero se recibieron {}",
        arity,
        args.len()
      ));
    }
    let mut arguments = vec![];
    let mut rest = vec![];
    for i in 0..args.len() {
      if i >= arity {
        rest.push(args[i].clone());
        continue;
      }
      arguments.push(args[i].clone());
    }
    if has_rest {
      arguments.push(Value::Object(rest.into()));
    }
    arguments.reverse();
    for arg in arguments {
      self.push(arg);
    }
    InterpretResult::Continue
  }
  pub fn run_instruction(&mut self) -> InterpretResult {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    //println!("{:<16} | {:?}", format!("{:?}", instruction), self.stack);

    let value: Value = match instruction {
      OpCode::OpAwait => {
        let value = self.pop();
        if value.is_promise() {
          let module = self.module.borrow().clone().unwrap();
          let blocking = BlockingThread::Await(value.as_promise());

          *module.sub_module.borrow_mut() = blocking;
        }
        value
      }
      OpCode::OpUnPromise => {
        let value = self.pop();
        match value.as_promise().get_data() {
          crate::bytecode::value::PromiseData::Err(e) => return InterpretResult::RuntimeError(e),
          crate::bytecode::value::PromiseData::Pending => {
            return InterpretResult::RuntimeError(format!(
              "El programa encontro un error de compilación en tiempo de ejecución (promesa no resuelta)"
            ))
          }
          crate::bytecode::value::PromiseData::Ok(v) => v.cloned(),
        }
      }
      OpCode::OpPromised => {
        // Debe existir el frame
        let frame = self.call_stack.pop().unwrap();
        let (async_thread, promise) = AsyncThread::from_frame(&*self, frame);
        self.vm.borrow_mut().push_sub_thread(async_thread);
        Value::Promise(promise)
      }
      OpCode::OpSetScope => {
        let value = self.pop();
        let vars = self.current_vars();
        value.set_scope(vars);
        value
      }
      OpCode::OpAt => Value::Iterator(self.pop().into()),
      OpCode::OpAsRef => Value::Ref(self.pop().into()),
      OpCode::OpCopy => {
        let value = self.pop();
        self.push(value.clone());
        value
      }
      OpCode::OpApproximate => {
        let value = self.pop();
        if !value.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '~{}'",
            value.get_type()
          ));
        }
        Value::Number(value.as_number().trunc())
      }
      OpCode::OpSetMember => {
        let value = self.pop();
        let key = self.pop();
        let object = self.pop();
        let is_instance = self.read() == 1u8;
        if is_instance {
          let key = key.as_string();
          if let Some(value) = object.set_instance_property(&key, value) {
            self.push(value);
            return InterpretResult::Continue;
          }
          let type_name = object.get_type();
          return InterpretResult::RuntimeError(format!(
            "No se pudo asignar la propiedad de instancia '{key}' de '{type_name}'"
          ));
        }
        if !object.is_object() {
          return InterpretResult::RuntimeError(format!(
            "Se esperaba un objeto para asignar la propiedad '{}' [3]",
            key.as_string()
          ));
        }
        let key = if object.is_array() {
          if !key.is_number() {
            return InterpretResult::RuntimeError(format!(
              "Se esperaba un indice de propiedad, pero se obtuvo '{}'",
              key.get_type()
            ));
          }
          let key = key.as_number();
          let index = match key {
            crate::bytecode::value::Number::Basic(n) => n,
            crate::bytecode::value::Number::Complex(_, _) => {
              return InterpretResult::RuntimeError(format!(
                "El indice no puede ser un valor complejo (asignar propiedad)"
              ));
            }
            crate::bytecode::value::Number::Infinity
            | crate::bytecode::value::Number::NaN
            | crate::bytecode::value::Number::NegativeInfinity => {
              return InterpretResult::RuntimeError(format!(
                "El indice no puede ser NaN o infinito (asignar propiedad)"
              ));
            }
          };
          if index.is_negative() {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser un numero entero positivo (asignar propiedad)"
            ));
          }
          if index.is_int() {
            index.to_string()
          } else {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser entero (asignar propiedad)"
            ));
          }
        } else {
          key.as_string()
        };
        match object.set_object_property(&key, value) {
          Some(value) => value,
          None => {
            let type_name = object.get_type();
            return InterpretResult::RuntimeError(if type_name == REF_TYPE {
              format!("Una referencia no puede ser modificada (asignar propiedad '{key}')",)
            } else {
              format!("No se pudo asignar la propiedad '{key}' a '{type_name}'",)
            });
          }
        }
      }
      OpCode::OpGetMember => {
        let key = self.pop();
        let object = self.pop();
        let is_instance = self.read() == 1u8;
        if is_instance {
          let key = key.as_string();
          let proto = self.vm.borrow().cache.proto.clone();
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
        let key = if object.is_array() {
          if !key.is_number() {
            return InterpretResult::RuntimeError(format!(
              "Se esperaba un indice de propiedad, pero se obtuvo '{}'",
              key.get_type()
            ));
          }
          let key = key.as_number();
          let index = match key {
            crate::bytecode::value::Number::Basic(n) => n,
            crate::bytecode::value::Number::Complex(_, _) => {
              return InterpretResult::RuntimeError(format!(
                "El indice no puede ser un valor complejo (obtener propiedad)"
              ));
            }
            crate::bytecode::value::Number::Infinity
            | crate::bytecode::value::Number::NaN
            | crate::bytecode::value::Number::NegativeInfinity => {
              return InterpretResult::RuntimeError(format!(
                "El indice no puede ser NaN o infinito (obtener propiedad)"
              ));
            }
          };
          if index.is_negative() {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser un numero entero positivo (obtener propiedad)"
            ));
          }
          if index.is_int() {
            index.to_string()
          } else {
            return InterpretResult::RuntimeError(format!(
              "El indice debe ser entero (obtener propiedad)"
            ));
          }
        } else {
          key.as_string()
        };
        match object.get_object_property(&key) {
          Some(value) => value,
          None => {
            let type_name = object.get_type();
            return InterpretResult::RuntimeError(format!(
              "No se pudo obtener la propiedad '{}' de '{}'",
              key, type_name
            ));
          }
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
          let m_path = self.module.borrow().clone().unwrap().path.clone();
          Path::new(&m_path)
            .parent()
            .unwrap()
            .join(path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
        }
        .replace("\\\\?\\", "");
        let proto = self.vm.borrow().cache.libs.clone();
        let value = libs(lib_name, proto, |path| {
          let module = Rc::new(RefCell::new(VM::resolve(
            self.vm.clone(),
            path,
            self.globals(),
          )));
          *self
            .module
            .borrow()
            .clone()
            .unwrap()
            .sub_module
            .borrow_mut() = BlockingThread::Module(module.clone());
          let x = module.borrow().clone().as_value();
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
        let name = self.read_string();
        let value = self.pop();
        let module = self.module.borrow().clone().unwrap().value.clone();
        if !module.is_object() {
          return InterpretResult::RuntimeError("Se esperaba un objeto como modulo".to_string());
        }
        match module.set_instance_property(&name, value.clone()) {
          Some(value) => value,
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo exportar la variable '{name}'"
            ))
          }
        }
      }
      OpCode::OpVarDecl => {
        let name = self.read_string();
        let value = self.pop();
        match self.declare(&name, value.clone(), false) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo declarar la variable '{name}'"
            ))
          }
          _ => value,
        }
      }
      OpCode::OpConstDecl => {
        let name = self.read_string();
        let value = self.pop();
        match self.declare(&name, value.clone(), true) {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo declarar la constante '{name}'"
            ))
          }
          _ => value,
        }
      }
      OpCode::OpDelVar => {
        let name = self.read_string();
        let vars = self.resolve(&name);
        if !vars.borrow().has(&name) {
          return InterpretResult::RuntimeError(format!(
            "No se pudo eliminar la variable '{name}'"
          ));
        }
        let value = vars.borrow_mut().remove(&name);
        match value {
          None => {
            return InterpretResult::RuntimeError(format!(
              "No se pudo eliminar la variable '{name}'"
            ))
          }
          Some(value) => value,
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
            Some(value) => value,
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
          self.push(Value::String(format!("{a}{b}")));
          return InterpretResult::Continue;
        }
        return InterpretResult::RuntimeError(format!(
          "No se pudo operar '{} + {}'",
          a.get_type(),
          b.get_type()
        ));
      }
      OpCode::OpSubtract => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} - {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a - b)
      }
      OpCode::OpMultiply => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} * {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a * b)
      }
      OpCode::OpDivide => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} / {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a / b)
      }
      OpCode::OpModulo => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} % {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number();
        let b = b.as_number();
        Value::Number(a % b)
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
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '-{}'",
            value.get_type()
          ));
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
          if a.is_nan() || b.is_nan() {
            Value::False
          } else if a == b {
            Value::True
          } else {
            Value::False
          }
        } else if a == b {
          Value::True
        } else {
          Value::False
        }
      }
      OpCode::OpGreaterThan => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return InterpretResult::RuntimeError(format!(
            "No se pudo operar '{} > {}'",
            a.get_type(),
            b.get_type()
          ));
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
