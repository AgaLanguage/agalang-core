use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::bytecode::compiler::{ChunkGroup, OpCode};
use crate::bytecode::libs::libs;
use crate::bytecode::stack::{CallFrame, InterpretResult, VarsManager};
use crate::bytecode::value::{Function, Object, Promise, PromiseData, Value, REF_TYPE};
use crate::bytecode::vm::VM;

#[derive(Clone, Debug)]
pub struct ModuleThread {
  path: String,
  value: Value,
  async_thread: Rc<RefCell<AsyncThread>>,
  status: InterpretResult,
  vm: Option<Rc<RefCell<VM>>>,
}
impl ModuleThread {
  pub fn new(path: &str) -> Rc<RefCell<Self>> {
    let (async_thread, _) = AsyncThread::new();

    let module = Rc::new(RefCell::new(Self {
      path: path.to_string(),
      async_thread: async_thread.clone(),
      value: Value::Object(Object::Map(HashMap::new().into(), HashMap::new().into())),
      status: InterpretResult::Continue,
      vm: None,
    }));
    async_thread.borrow_mut().set_module(module.clone());
    module
  }
  pub fn as_value(self) -> Value {
    self.value
  }
  pub fn run_instruction(&self) -> InterpretResult {
    if !matches!(self.status, InterpretResult::Continue | InterpretResult::Ok) {
      return self.status.clone();
    }
    let code = self.async_thread.borrow().thread.borrow_mut().peek();
    match code {
      OpCode::OpImport => {
        let thread = self.async_thread.borrow().thread.clone();
        thread.borrow_mut().read();
        let module = thread.borrow_mut().pop();
        let path = module.as_string();
        let meta_byte = thread.borrow_mut().read();
        let name_byte = thread.borrow_mut().read();
        let _is_lazy = (meta_byte & 0b10) == 0b10;
        let alias = (meta_byte & 0b01) == 0b01;

        let lib_name = if path.starts_with(":") {
          path
        } else {
          Path::new(&self.path)
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
        let proto = self.get_vm().borrow().cache.libs.clone();
        let value = libs(lib_name, proto, |path| {
          let module = VM::resolve(self.get_vm(), path, thread.borrow_mut().globals());
          *self.async_thread.borrow().await_thread.borrow_mut() =
            BlockingThread::Module(module.clone());
          let x = module.borrow().clone().as_value();
          x
        });
        if alias {
          let name = thread
            .borrow_mut()
            .current_chunk()
            .read_constant(name_byte)
            .as_string();
          thread.borrow_mut().declare(&name, value, true);
        }
        thread.borrow_mut().push(module);
        InterpretResult::Continue
      }
      OpCode::OpExport => {
        let thread = self.async_thread.borrow().thread.clone();
        thread.borrow_mut().read();
        let name = thread.borrow_mut().read_string();
        let value = thread.borrow_mut().pop();
        if !self.value.is_object() {
          return InterpretResult::RuntimeError("Se esperaba un objeto como modulo".to_string());
        }
        match self.value.set_instance_property(&name, value.clone()) {
          Some(value) => {
            thread.borrow_mut().push(value);
            InterpretResult::Continue
          }
          None => {
            InterpretResult::RuntimeError(format!("No se pudo exportar la variable '{name}'"))
          }
        }
      }
      _ => self.async_thread.borrow().run_instruction_as_module(),
    }
  }
  pub fn push_call(&self, frame: CallFrame) {
    self.async_thread.borrow().push_call(frame)
  }
  pub fn set_vm(&mut self, vm: Rc<RefCell<VM>>) {
    self.vm = Some(vm);
  }
  pub fn set_status(&mut self, status: InterpretResult) {
    self.status = status;
  }
  fn get_vm(&self) -> Rc<RefCell<VM>> {
    self.vm.clone().unwrap()
  }
  pub fn get_async(&self) -> Rc<RefCell<AsyncThread>> {
    self.async_thread.clone()
  }
}

#[derive(Clone, Debug, Default)]
enum BlockingThread {
  #[default]
  Void,
  Module(Rc<RefCell<ModuleThread>>),
  Await(Promise),
}
impl BlockingThread {
  fn run_instruction(&self) -> InterpretResult {
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
pub struct AsyncThread {
  promise: Promise,
  thread: Rc<RefCell<Thread>>,
  await_thread: Rc<RefCell<BlockingThread>>,
  module: Option<Rc<RefCell<ModuleThread>>>,
}
impl AsyncThread {
  fn set_module(&mut self, module: Rc<RefCell<ModuleThread>>) {
    self.module = Some(module);
  }
  fn pop(&self) -> Value {
    self.thread.borrow_mut().pop()
  }
  fn push(&self, value: Value) {
    self.thread.borrow_mut().push(value)
  }
  fn from_frame(frame: CallFrame) -> (Rc<RefCell<Self>>, Promise) {
    let (thread, promise) = Self::new();
    thread.borrow().push_call(frame);
    (thread, promise)
  }
  fn new() -> (Rc<RefCell<Self>>, Promise) {
    let original_promise = Promise::new();
    let promise = original_promise.clone();
    let thread = Thread::new();
    let async_thread = Rc::new(RefCell::new(Self {
      promise,
      thread: thread.clone(),
      await_thread: Default::default(),
      module: Default::default(),
    }));
    thread.borrow_mut().set_async(async_thread.clone());
    (async_thread, original_promise)
  }
  fn push_call(&self, frame: CallFrame) {
    let sub_thread = self.await_thread.borrow().clone();
    if let BlockingThread::Module(sub_module) = sub_thread {
      return sub_module.borrow_mut().push_call(frame);
    }
    self.thread.borrow_mut().push_call(frame)
  }
  fn run_instruction(&self) -> InterpretResult {
    let code = self.thread.borrow_mut().peek();
    match code {
      OpCode::OpReturn => {
        self.thread.borrow_mut().read();
        let value = self.thread.borrow_mut().pop();
        self.promise.set_value(value);
        return InterpretResult::Ok;
      }
      OpCode::OpAwait => {
        self.thread.borrow_mut().read();
        let value = self.pop();
        if value.is_promise() {
          let blocking = BlockingThread::Await(value.as_promise());
          *self.await_thread.borrow_mut() = blocking;
        }
        self.push(value);
        return InterpretResult::Continue;
      }
      _ => {}
    };
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
  pub fn run_instruction_as_module(&self) -> InterpretResult {
    let sub_module = self.await_thread.borrow().clone();

    if matches!(sub_module, BlockingThread::Void) {
      return self.run_instruction();
    }

    let data = self.await_thread.borrow().run_instruction();
    if !matches!(data, InterpretResult::Continue) {
      *self.await_thread.borrow_mut() = BlockingThread::Void;
      return InterpretResult::Continue;
    }

    data
  }
  pub fn get_thread(&self) -> Rc<RefCell<Thread>> {
    self.thread.clone()
  }
}

#[derive(Clone, Debug)]
pub struct Thread {
  stack: Vec<Value>,
  call_stack: Vec<CallFrame>,
  async_thread: Option<Rc<RefCell<AsyncThread>>>,
}

impl Thread {
  fn new() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self {
      stack: vec![],
      call_stack: vec![],
      async_thread: None,
    }))
  }
  fn set_async(&mut self, async_thread: Rc<RefCell<AsyncThread>>) {
    self.async_thread = Some(async_thread);
  }
  pub fn get_stack(&self) -> &Vec<Value> {
    &self.stack
  }
  pub fn get_calls(&self) -> &Vec<CallFrame> {
    &self.call_stack
  }
  pub fn clear_stack(&mut self) {
    self.stack.clear();
  }
  fn push_call(&mut self, frame: CallFrame) {
    self.call_stack.push(frame);
  }
  fn current_frame(&mut self) -> &mut CallFrame {
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
    let mut vars = self.globals();
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
  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }
  fn read(&mut self) -> u8 {
    self.current_frame().read()
  }
  fn peek(&mut self) -> OpCode {
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
        let value = func(this, args, self);
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
  fn run_instruction(&mut self) -> InterpretResult {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    let value: Value = match instruction {
      OpCode::OpImport | OpCode::OpExport => {
        return InterpretResult::CompileError(format!("Solo un modulo puede exportar o importar"))
      }
      OpCode::OpAwait => {
        return InterpretResult::CompileError(format!("Solo un hilo asincrono puede esperar"))
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
        let (async_thread, promise) = AsyncThread::from_frame(frame);
        let current_async = self.async_thread.clone().unwrap();
        let module = current_async.borrow().module.clone().unwrap();
        async_thread.borrow_mut().set_module(module.clone());
        let vm = module.borrow().get_vm();
        vm.borrow_mut().push_sub_thread(async_thread);
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
          let async_thread = self.async_thread.clone().unwrap();
          let module = async_thread.borrow().module.clone().unwrap();
          let vm = module.borrow().get_vm();
          let proto = vm.borrow().cache.proto.clone();
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
