use std::collections::HashMap;
use std::path::Path;

use super::VM;
use crate::compiler::{Function, LazyValue, Number, Object, OpCode, Promise, PromiseData, Value};
use crate::functions_names::CONSTRUCTOR;
use crate::interpreter::stack::{CallFrame, InterpretResult};
use crate::interpreter::VarsManager;
use crate::MultiRefHash;

#[derive(Clone, Debug)]
pub struct ModuleThread {
  path: String,
  value: Value,
  async_thread: MultiRefHash<AsyncThread>,
  status: InterpretResult,
  vm: Option<MultiRefHash<VM>>,
}
impl ModuleThread {
  pub fn new(path: &str) -> MultiRefHash<Self> {
    let (async_thread, _) = AsyncThread::new();

    let module: MultiRefHash<ModuleThread> = Self {
      path: path.to_string(),
      async_thread: async_thread.clone(),
      value: Value::Object(Object::Map(
        HashMap::new().into(),
        crate::compiler::Instance::new(format!("<{path}>")).into(),
      )),
      status: InterpretResult::Continue,
      vm: None,
    }
    .into();
    async_thread.write().set_module(module.clone());
    module
  }
  pub fn as_value(self) -> Value {
    self.value
  }
  pub fn run_instruction(&self) -> InterpretResult {
    if !matches!(self.status, InterpretResult::Continue | InterpretResult::Ok) {
      return self.status.clone();
    }
    let code = self.async_thread.read().thread.write().peek();
    match code {
      OpCode::OpImport => {
        let thread = self.async_thread.read().thread.clone();
        thread.write().read();
        let module = thread.write().pop();
        let path = module.as_string();
        let meta_byte = thread.write().read();
        let name_byte = thread.write().read();
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
        let proto = self.get_vm().read().cache.libs.clone();
        let value = crate::interpreter::libs::libs(lib_name, proto, |path| {
          let module = VM::resolve(self.get_vm(), path, thread.write().globals());
          *self.async_thread.read().await_thread.write() = BlockingThread::Module(module.clone());
          let x = module.read().clone().as_value();
          x
        });
        if alias {
          let name = thread
            .read()
            .current_chunk()
            .read()
            .read_constant(name_byte)
            .as_string();
          thread.write().declare(&name, value, true);
        }
        thread.write().push(module);
        InterpretResult::Continue
      }
      OpCode::OpExport => {
        let thread = self.async_thread.read().thread.clone();
        thread.write().read();
        let name = thread.write().read_string();
        let value = thread.write().pop();
        if !self.value.is_object() {
          return InterpretResult::RuntimeError("Se esperaba un objeto como modulo".to_string());
        }
        match self.value.set_instance_property(
          &name,
          value.clone(),
          true,
          false,
          &*self.get_async().read().get_thread().read(),
        ) {
          Some(value) => {
            thread.write().push(value);
            InterpretResult::Continue
          }
          None => {
            InterpretResult::RuntimeError(format!("No se pudo exportar la variable '{name}'"))
          }
        }
      }
      _ => self.async_thread.read().run_instruction(),
    }
  }
  pub fn push_call(&self, frame: CallFrame) {
    self.async_thread.read().push_call(frame)
  }
  pub fn set_vm(&mut self, vm: MultiRefHash<VM>) {
    self.vm = Some(vm);
  }
  pub fn set_status(&mut self, status: InterpretResult) {
    self.status = status;
  }
  pub fn get_vm(&self) -> MultiRefHash<VM> {
    self.vm.clone().unwrap()
  }
  pub fn get_async(&self) -> MultiRefHash<AsyncThread> {
    self.async_thread.clone()
  }
  pub fn get_process_manager(
    &self,
  ) -> MultiRefHash<crate::interpreter::vm::process::ProcessManager> {
    self.get_vm().read().get_process_manager()
  }
}

#[derive(Clone, Debug, Default)]
enum TryCatchState {
  #[default]
  Trying,
  Error(String),
  Catching,
  Ending,
}

#[derive(Clone, Debug, Default)]
enum BlockingThread {
  #[default]
  Void,
  Module(MultiRefHash<ModuleThread>),
  Await(Promise),
  TryCatch {
    try_thread: MultiRefHash<AsyncThread>,
    catch_thread: MultiRefHash<AsyncThread>,
    state: MultiRefHash<TryCatchState>,
  },
  Lazy(LazyValue, MultiRefHash<AsyncThread>),
}
impl BlockingThread {
  fn run_instruction(&self) -> InterpretResult {
    match self {
      // nada que ejecutar
      Self::Void => InterpretResult::Ok,
      Self::Module(ref_cell) => ref_cell.read().run_instruction(),
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
      Self::TryCatch {
        try_thread,
        catch_thread,
        state,
      } => {
        let current_state = state.read().clone();
        match current_state {
          TryCatchState::Trying => {
            let result = try_thread.read().run_instruction();
            match result {
              InterpretResult::Ok => {
                *state.write() = TryCatchState::Ending;
                InterpretResult::Continue
              }
              InterpretResult::RuntimeError(message) => {
                *state.write() = TryCatchState::Error(message);
                InterpretResult::Continue
              }
              result => result,
            }
          }
          TryCatchState::Error(message) => {
            catch_thread.read().push(Value::String(message.clone()));
            *state.write() = TryCatchState::Catching;
            InterpretResult::Continue
          }
          TryCatchState::Catching => {
            let result = catch_thread.read().run_instruction();
            match result {
              InterpretResult::Ok => {
                *state.write() = TryCatchState::Ending;
                InterpretResult::Continue
              }
              result => result,
            }
          }
          _ => InterpretResult::Ok,
        }
      }
      Self::Lazy(lazy, thread) => {
        let byte = thread.read().get_thread().write().peek();
        match byte {
          OpCode::OpReturn => {
            let value = thread.read().pop();
            thread.read().push(Value::Never);
            thread.read().run_instruction();
            lazy.set(value);
            InterpretResult::Ok
          }
          _ => thread.read().run_instruction(),
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct AsyncThread {
  promise: Promise,
  thread: MultiRefHash<Thread>,
  await_thread: MultiRefHash<BlockingThread>,
  module: Option<MultiRefHash<ModuleThread>>,
}
impl AsyncThread {
  pub fn is_waiting(&self) -> bool {
    let bloking_thread = &*self.await_thread.read();
    match bloking_thread {
      BlockingThread::Await(p) => match p.get_data() {
        PromiseData::Pending => true,
        _ => false,
      },
      _ => false,
    }
  }
  pub fn set_module(&mut self, module: MultiRefHash<ModuleThread>) {
    self.module = Some(module);
  }
  pub fn get_module(&self) -> MultiRefHash<ModuleThread> {
    self.module.clone().unwrap()
  }
  fn pop(&self) -> Value {
    self.thread.write().pop()
  }
  pub fn push(&self, value: Value) {
    self.thread.write().push(value)
  }
  pub fn from_frame(frame: CallFrame) -> (MultiRefHash<Self>, Promise) {
    let (thread, promise) = Self::new();
    thread.read().push_call(frame);
    (thread, promise)
  }
  pub fn new() -> (MultiRefHash<Self>, Promise) {
    let original_promise = Promise::new();
    let promise = original_promise.clone();
    let thread = Thread::new();
    let async_thread: MultiRefHash<AsyncThread> = Self {
      promise,
      thread: thread.clone(),
      await_thread: Default::default(),
      module: Default::default(),
    }
    .into();
    thread.write().set_async(async_thread.clone());
    (async_thread, original_promise)
  }
  fn push_call(&self, frame: CallFrame) {
    let sub_thread = self.await_thread.read().clone();
    if let BlockingThread::Module(sub_module) = sub_thread {
      return sub_module.write().push_call(frame);
    }
    self.thread.write().push_call(frame)
  }
  pub fn simple_run_instruction(&self, contain_error: bool) -> InterpretResult {
    let code = self.thread.write().peek();
    match code {
      OpCode::OpAwait => {
        self.thread.write().read();
        let value = self.pop();
        if value.is_promise() {
          let blocking = BlockingThread::Await(value.as_promise());
          *self.await_thread.write() = blocking;
        }
        self.push(value);
        InterpretResult::Continue
      }
      _ => {
        let result = self.thread.write().run_instruction();
        match result {
          InterpretResult::RuntimeError(err) => {
            self.promise.set_err(err.clone());
            if contain_error {
              // Este es un error de la promesa, no de el programa
              InterpretResult::Ok
            } else {
              InterpretResult::RuntimeError(err)
            }
          }
          InterpretResult::Ok => {
            self.promise.set_value(self.pop());
            InterpretResult::Ok
          }
          result => result,
        }
      }
    }
  }
  pub fn run_instruction(&self) -> InterpretResult {
    let sub_module = self.await_thread.read().clone();

    if matches!(sub_module, BlockingThread::Void) {
      return self.simple_run_instruction(false);
    }

    let data = self.await_thread.read().run_instruction();
    if !matches!(data, InterpretResult::Continue) {
      *self.await_thread.write() = BlockingThread::Void;
      return InterpretResult::Continue;
    }

    data
  }
  pub fn get_thread(&self) -> MultiRefHash<Thread> {
    self.thread.clone()
  }
}

#[derive(Clone, Debug)]
pub struct Thread {
  stack: MultiRefHash<Vec<Value>>,
  call_stack: MultiRefHash<Vec<CallFrame>>,
  async_thread: MultiRefHash<Option<MultiRefHash<AsyncThread>>>,
}

impl Thread {
  fn new() -> MultiRefHash<Self> {
    Self {
      stack: vec![].into(),
      call_stack: vec![].into(),
      async_thread: None.into(),
    }
    .into()
  }
  fn set_async(&self, async_thread: MultiRefHash<AsyncThread>) {
    *self.async_thread.write() = Some(async_thread);
  }
  pub fn get_async(&self) -> MultiRefHash<AsyncThread> {
    self.async_thread.unwrap()
  }
  pub fn get_stack(&self) -> std::sync::RwLockReadGuard<Vec<Value>> {
    self.stack.read()
  }
  pub fn get_calls(&self) -> std::sync::RwLockReadGuard<Vec<CallFrame>> {
    self.call_stack.read()
  }
  pub fn clear_stack(&mut self) {
    self.stack.clear();
  }
  pub fn push_call(&mut self, frame: CallFrame) {
    self.call_stack.push(frame);
  }
  fn with_current_frame_mut<R>(&self, callback: impl FnOnce(&mut CallFrame) -> R) -> R {
    callback(self.call_stack.write().last_mut().unwrap())
  }
  fn current_chunk(&self) -> MultiRefHash<crate::compiler::ChunkGroup> {
    self.call_stack.read().last().unwrap().current_chunk()
  }
  fn globals(&self) -> MultiRefHash<VarsManager> {
    self.call_stack.read().last().unwrap().globals()
  }
  pub fn current_vars(&self) -> MultiRefHash<VarsManager> {
    self.call_stack.read().last().unwrap().current_vars()
  }
  fn resolve(&mut self, name: &str) -> MultiRefHash<VarsManager> {
    let mut vars = self.globals();
    for call in self.call_stack.read().iter() {
      let local_vars = call.resolve_vars(name);
      if local_vars.read().has(name) {
        vars = local_vars;
      }
    }
    vars
  }
  fn declare(&self, name: &str, value: Value, is_constant: bool) -> Option<Value> {
    self
      .current_vars()
      .write()
      .declare(name, value, is_constant)
  }
  fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
    self.resolve(name).write().assign(name, value)
  }
  fn get(&mut self, name: &str) -> Option<Value> {
    self.resolve(name).read().get(name).cloned()
  }
  pub fn runtime_error(&mut self, message: &str) {
    let binding = self.call_stack.read();
    let frame = binding.last().unwrap();
    let line = frame.current_line().to_string();
    let name = frame.to_string();
    eprintln!("[linea {line}] en {name}\n{message}");

    drop(binding);

    self.reset_stack();
  }
  fn reset_stack(&mut self) {
    self.stack.clear();
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Value {
    self.stack.pop()
  }
  fn init(&self, value: &Value) {
    match value {
      Value::Lazy(lazy) => {
        let (thread, _) = AsyncThread::new();
        let once = lazy.get_once();
        let vars = VarsManager::crate_child(
          once
            .read()
            .get_scope()
            .unwrap_or_else(|| self.current_vars()),
        );
        thread
          .read()
          .push_call(CallFrame::new(once, vec![vars.into()]));
        *self.get_async().read().await_thread.write() = BlockingThread::Lazy(lazy.clone(), thread);
      }
      _ => {}
    };
  }
  fn read(&mut self) -> u8 {
    self.with_current_frame_mut(|frame| frame.read())
  }
  pub fn peek(&self) -> OpCode {
    self.call_stack.read().last().unwrap().peek().into()
  }
  fn read_constant(&mut self) -> Value {
    let constant_index = self.read();
    self
      .current_chunk()
      .read()
      .read_constant(constant_index)
      .clone()
  }
  fn read_string(&mut self) -> String {
    self.read_constant().as_string()
  }
  fn read_short(&mut self) -> u16 {
    let a = self.read() as u16;
    let b = self.read() as u16;
    (a << 8) | b
  }
  pub fn call_function(
    &mut self,
    this: Value,
    fun: MultiRefHash<Function>,
    args: Vec<Value>,
  ) -> Result<InterpretResult, String> {
    let fun_clone = fun.clone();
    let function = fun_clone.read();

    let (arity, has_rest, is_async) = match &*function {
      Function::Function {
        arity,
        has_rest,
        is_async,
        ..
      } => (
        if *has_rest { arity - 1 } else { *arity },
        *has_rest,
        *is_async,
      ),
      Function::Script { .. } => (0, false, false),
      Function::Native {
        func, custom_data, ..
      } => {
        let value = func(this, args, self, custom_data.clone());
        self.push(value?);
        return Ok(InterpretResult::Continue);
      }
    };
    // En el caso de que la funcion no tenga un scope definido, se usa el scope actual (esto deberia de pasar)
    let vars = function.get_scope().unwrap_or_else(|| self.current_vars());
    let locals = vec![VarsManager::crate_child(vars.clone())
      .set_this(this.clone())
      .into()];
    if is_async {}
    self.call_stack.push(CallFrame::new(fun, locals));

    if arity > args.len() {
      if arity == 1 && args.len() == 0 {
        return Err("Se esperaba llamar una funcion con un argumento".into());
      }
      return Err(format!(
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
    Ok(InterpretResult::Continue)
  }
  fn call_value(
    &mut self,
    this: Value,
    callee: Value,
    arity: usize,
  ) -> Result<InterpretResult, String> {
    let mut args = vec![];
    for _ in 0..arity {
      let value = self.pop();
      if !value.is_iterator() {
        args.push(value);
        continue;
      }
      let mut list = value.as_strict_array(self)?;
      loop {
        match list.pop() {
          Some(value) => args.push(value),
          None => break,
        }
      }
    }
    args.reverse();

    if callee.is_number() {
      if arity != 1 || args.len() != 1 {
        return Err("Solo se puede multiplicar un numero (llamada)".into());
      }
      let arg = args.get(0).unwrap();
      if !arg.is_number() {
        return Err("Solo se pueden multiplicar numeros (llamada)".into());
      }
      let num = callee.as_number()?;
      let arg = arg.as_number()?;
      let value = Value::Number(num * arg);
      self.push(value);
      return Ok(InterpretResult::Continue);
    }
    if callee.is_class() {
      let class = callee.as_class();
      let this = this
        .as_map()
        .1
        .on_ok(|instance| {
          if class.read().is_instance(instance) {
            Some(this)
          } else {
            None
          }
        })
        .unwrap_or_else(|| class.read().make_instance());
      let constructor = class.read().get_instance_property(CONSTRUCTOR);
      if let Some(Value::Object(Object::Function(fun))) = constructor {
        self.call_function(this.clone(), fun, args)?;
      } else if let Some(_) = constructor {
        return Err("Se esperaba llamar un constructor".into());
      } else {
        self.push(this);
      }
      return Ok(InterpretResult::Continue);
    }
    if callee.is_function() {
      return self.call_function(this, callee.as_function(), args);
    }
    return Err("Se esperaba llamar una funcion".into());
  }
  fn simple_run_instruction(&mut self) -> Result<InterpretResult, String> {
    let byte_instruction = self.read();
    let instruction: OpCode = byte_instruction.into();

    let value: Value = match instruction {
      OpCode::OpThrow => return Err(self.pop().as_string()),
      OpCode::OpTry => {
        let catch_block = self.pop().as_function();
        let try_block = self.pop().as_function();

        let module = self.get_async().read().get_module();

        let (try_thread, _) = AsyncThread::new();
        try_thread.write().set_module(module.clone());
        try_thread.read().push_call(CallFrame::new(
          try_block,
          vec![VarsManager::crate_child(self.current_vars()).into()],
        ));

        let (catch_thread, _) = AsyncThread::new();
        catch_thread.write().set_module(module);
        catch_thread.read().push_call(CallFrame::new(
          catch_block,
          vec![VarsManager::crate_child(self.current_vars()).into()],
        ));
        *self.get_async().read().await_thread.write() = BlockingThread::TryCatch {
          try_thread,
          catch_thread,
          state: Default::default(),
        };
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpImport | OpCode::OpExport => {
        return Err(format!("Solo un modulo puede exportar o importar"))
      }
      OpCode::OpAwait => return Err(format!("Solo un hilo asincrono puede esperar")),
      OpCode::OpUnPromise => {
        let value = self.pop();
        match value.as_promise().get_data() {
          PromiseData::Err(e) => return Err(e),
          PromiseData::Pending => {
            return Err(format!(
              "El programa encontro un error de compilación en tiempo de ejecución (promesa no resuelta)"
            ))
          }
          PromiseData::Ok(v) => v.cloned(),
        }
      }
      OpCode::OpExtendClass => {
        let parent_class = match self.pop() {
          Value::Object(Object::Class(class)) => class.cloned(),
          value => {
            return Err(format!(
              "No se puede usar '{}' para extender una clase",
              value.get_type()
            ))
          }
        };
        let value = self.pop();
        value.as_class().read().set_parent(parent_class);
        value
      }
      OpCode::OpInClass => {
        let class = self.pop().as_class();
        let value = self.pop();
        value.set_in_class(class);
        value
      }
      OpCode::OpGetInstance => self.pop().as_class().read().make_instance(),
      OpCode::OpPromised => {
        // Debe existir el frame
        let frame = self.call_stack.pop();
        let (async_thread, promise) = AsyncThread::from_frame(frame);
        let current_async = self.async_thread.clone().unwrap();
        let module = current_async.read().module.clone().unwrap();
        async_thread.write().set_module(module.clone());
        module
          .read()
          .get_vm()
          .read()
          .get_process_manager()
          .read()
          .push_sub_thread(async_thread);
        self.push(Value::Promise(promise));
        return if self.call_stack.len() > 0 {
          Ok(InterpretResult::Continue)
        } else {
          Ok(InterpretResult::Ok)
        };
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
          return Err(format!("No se pudo operar '~{}'", value.get_type()));
        }
        Value::Number(value.as_number()?.trunc())
      }
      OpCode::OpSetMember => {
        let value = self.pop();
        let key = self.pop();
        let object = self.pop();
        let meta = self.read();
        let is_instance = (meta & 0b001) != 0;
        let is_public = (meta & 0b010) != 0;
        let is_class_decl = (meta & 0b100) != 0;
        if is_instance {
          let key = key.as_string();
          use crate::util::OnError;
          let value = object
            .set_instance_property(&key, value.clone(), is_public, is_class_decl, self)
            .on_error(|_| {
              format!("Las asignaciones de instancia solo estan permitidas dentro de clases: {key}")
            })?;
          self.push(value);
          return Ok(InterpretResult::Continue);
        }
        if !object.is_object() {
          return Err(format!(
            "Se esperaba un objeto para asignar la propiedad '{}' [3]",
            key.as_string()
          ));
        }
        let key = if object.is_array() {
          if !key.is_number() {
            return Err(format!(
              "Se esperaba un indice de propiedad, pero se obtuvo '{}'",
              key.get_type()
            ));
          }
          let key = key.as_number()?;
          let index = match key {
            Number::Basic(n) => n,
            Number::Complex(_, _) => {
              return Err(format!(
                "El indice no puede ser un valor complejo (asignar propiedad)"
              ));
            }
            Number::Infinity | Number::NaN | Number::NegativeInfinity => {
              return Err(format!(
                "El indice no puede ser NaN o infinito (asignar propiedad)"
              ));
            }
          };
          if index.is_negative() {
            return Err(format!(
              "El indice debe ser un numero entero positivo (asignar propiedad)"
            ));
          }
          if index.is_int() {
            index.to_string()
          } else {
            return Err(format!("El indice debe ser entero (asignar propiedad)"));
          }
        } else {
          key.as_string()
        };
        match object.set_object_property(&key, value) {
          Some(value) => value,
          None => {
            let type_name = object.get_type();
            return Err(if type_name == crate::compiler::REF_TYPE {
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
          if let Some(value) = object.get_instance_property(&key, self) {
            self.init(&value);
            self.push(value);
            return Ok(InterpretResult::Continue);
          }
          let type_name = object.get_type();
          return Err(format!(
            "No se pudo obtener la propiedad de instancia '{key}' de '{type_name}'"
          ));
        }
        if !object.is_object() {
          return Err(format!(
            "Se esperaba un objeto para obtener la propiedad '{}' [3]",
            key.as_string()
          ));
        }
        let key = if object.is_array() {
          if !key.is_number() {
            return Err(format!(
              "Se esperaba un indice de propiedad, pero se obtuvo '{}'",
              key.get_type()
            ));
          }
          let key = key.as_number()?;
          let index = match key {
            Number::Basic(n) => n,
            Number::Complex(_, _) => {
              return Err(format!(
                "El indice no puede ser un valor complejo (obtener propiedad)"
              ));
            }
            Number::Infinity | Number::NaN | Number::NegativeInfinity => {
              return Err(format!(
                "El indice no puede ser NaN o infinito (obtener propiedad)"
              ));
            }
          };
          if index.is_negative() {
            return Err(format!(
              "El indice debe ser un numero entero positivo (obtener propiedad)"
            ));
          }
          if index.is_int() {
            index.to_string()
          } else {
            return Err(format!("El indice debe ser entero (obtener propiedad)"));
          }
        } else {
          key.as_string()
        };
        match object.get_object_property(&key) {
          Some(value) => {
            self.init(&value);
            value
          }
          None => {
            let type_name = object.get_type();
            return Err(format!(
              "No se pudo obtener la propiedad '{}' de '{}'",
              key, type_name
            ));
          }
        }
      }
      OpCode::OpConstant => self.read_constant(),
      OpCode::OpJumpIfFalse => {
        let jump = self.read_short() as usize;
        let value = self.pop().as_boolean()?;
        if value == false {
          self.with_current_frame_mut(|frame| frame.advance(jump));
        }
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpArgDecl => {
        let name = self.read_string();
        let value = self.pop();
        return match self.declare(&name, value.clone(), true) {
          None => Err(format!("No se pudo declarar la variable '{name}'")),
          _ => Ok(InterpretResult::Continue),
        };
      }
      OpCode::OpJump => {
        let jump = self.read_short() as usize;

        self.with_current_frame_mut(|frame| frame.advance(jump));
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpLoop => {
        let offset = self.read_short() as usize;

        self.with_current_frame_mut(|frame| frame.back(offset));
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpRemoveLocals => {
        self.with_current_frame_mut(|frame| frame.pop_vars());
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpNewLocals => {
        self.with_current_frame_mut(|frame| frame.add_vars());
        return Ok(InterpretResult::Continue);
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
          None => return Err(format!("No se pudo declarar la variable '{name}'")),
          _ => value,
        }
      }
      OpCode::OpConstDecl => {
        let name = self.read_string();
        let value = self.pop();
        match self.declare(&name, value.clone(), true) {
          None => return Err(format!("No se pudo declarar la constante '{name}'")),
          _ => value,
        }
      }
      OpCode::OpDelVar => {
        let name = self.read_string();
        let vars = self.resolve(&name);
        if !vars.read().has(&name) {
          return Err(format!("No se pudo eliminar la variable '{name}'"));
        }
        let value = vars.write().remove(&name);
        match value {
          None => return Err(format!("No se pudo eliminar la variable '{name}'")),
          Some(value) => value,
        }
      }
      OpCode::OpGetVar => {
        let name = self.read_string();
        let value = {
          let v = self.get(&name);
          match v {
            None => return Err(format!("No se pudo obtener la variable '{name}'")),
            Some(value) => value,
          }
        };
        self.init(&value);
        value
      }
      OpCode::OpSetVar => {
        let name = self.read_string();
        let value = self.pop();
        match self.assign(&name, value.clone()) {
          None => return Err(format!("No se pudo re-asignar la variable '{name}'")),
          _ => value,
        }
      }
      OpCode::OpPop => {
        self.pop();
        return Ok(InterpretResult::Continue);
      }
      OpCode::OpAdd => {
        let b = self.pop();
        let a = self.pop();
        if a.is_number() && b.is_number() {
          let a = a.as_number()?;
          let b = b.as_number()?;
          self.push(Value::Number(a + b));
          return Ok(InterpretResult::Continue);
        }
        if a.is_string() || b.is_string() {
          let a = a.as_string();
          let b = b.as_string();
          self.push(Value::String(format!("{a}{b}")));
          return Ok(InterpretResult::Continue);
        }
        return Err(format!(
          "No se pudo operar '{} + {}'",
          a.get_type(),
          b.get_type()
        ));
      }
      OpCode::OpSubtract => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return Err(format!(
            "No se pudo operar '{} - {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number()?;
        let b = b.as_number()?;
        Value::Number(a - b)
      }
      OpCode::OpMultiply => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return Err(format!(
            "No se pudo operar '{} * {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number()?;
        let b = b.as_number()?;
        Value::Number(a * b)
      }
      OpCode::OpDivide => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return Err(format!(
            "No se pudo operar '{} / {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number()?;
        let b = b.as_number()?;
        Value::Number(a / b)
      }
      OpCode::OpModulo => {
        let b = self.pop();
        let a = self.pop();
        if !a.is_number() || !b.is_number() {
          return Err(format!(
            "No se pudo operar '{} % {}'",
            a.get_type(),
            b.get_type()
          ));
        }
        let a = a.as_number()?;
        let b = b.as_number()?;
        Value::Number(a % b)
      }
      OpCode::OpOr => {
        let b = self.pop();
        let a = self.pop();
        if a.as_boolean()? {
          a
        } else {
          b
        }
      }
      OpCode::OpAnd => {
        let b = self.pop();
        let a = self.pop();
        if !a.as_boolean()? {
          a
        } else {
          b
        }
      }
      OpCode::OpNegate => {
        let value = self.pop();
        if !value.is_number() {
          return Err(format!("No se pudo operar '-{}'", value.get_type()));
        }
        Value::Number(-value.as_number()?)
      }
      OpCode::OpNot => {
        let value = self.pop().as_boolean()?;
        let value = if value { Value::False } else { Value::True };
        value
      }
      OpCode::OpAsBoolean => {
        let value = self.pop().as_boolean()?;
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
        if self.call_stack.len() == 0 {
          return Ok(InterpretResult::Ok);
        }
        self.pop()
      }
      OpCode::OpEquals => {
        let b = self.pop();
        let a = self.pop();
        if a.is_number() && b.is_number() {
          let a = a.as_number()?;
          let b = b.as_number()?;
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
          return Err(format!(
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
          return Err(format!(
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
      OpCode::OpNull => return Err(format!("Byte invalido {}", byte_instruction)),
    };
    self.push(value);
    return Ok(InterpretResult::Continue);
  }
  fn run_instruction(&mut self) -> InterpretResult {
    match self.simple_run_instruction() {
      Ok(result) => result,
      Err(error) => InterpretResult::RuntimeError(error),
    }
  }
}
