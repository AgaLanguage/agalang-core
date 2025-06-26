use std::{collections::VecDeque, fmt::Debug, sync::RwLock};

use crate::{
  interpreter::{stack::InterpretResult, Thread},
  MultiRefHash,
};

use super::thread::{AsyncThread, ModuleThread};

#[derive(Debug)]
pub struct ProcessManager {
  main: MultiRefHash<ModuleThread>,
  sub_threads: RwLock<VecDeque<MultiRefHash<AsyncThread>>>,
  waiting_threads: RwLock<VecDeque<MultiRefHash<AsyncThread>>>,
  interrupt_threads: RwLock<VecDeque<MultiRefHash<AsyncThread>>>,
}

impl ProcessManager {
  pub fn new(main: MultiRefHash<ModuleThread>) -> Self {
    Self {
      main,
      sub_threads: Default::default(),
      waiting_threads: Default::default(),
      interrupt_threads: Default::default(),
    }
  }
  pub fn as_value(&self) -> crate::compiler::Value {
    self.main.read().clone().as_value()
  }
  pub fn get_root_thread(&self) -> MultiRefHash<Thread> {
    self.main.read().get_async().read().get_thread()
  }
  pub fn run_instruction(&self) -> InterpretResult {
    // Ejecuta una instruccion de cada hilo de interrupcion, por ser prioritarios
    self.run_interrupt_threads();
    self.poll_waiting_threads();

    let first_item = self.sub_threads.write().unwrap().pop_front();
    if let Some(thread) = first_item {
      let response = thread.read().simple_run_instruction(true);
      if thread.read().is_waiting() {
        self
          .waiting_threads
          .write()
          .unwrap()
          .push_back(thread.clone());
      } else if matches!(response, InterpretResult::Continue) {
        self.sub_threads.write().unwrap().push_back(thread);
      }
    }
    // El hilo debe ejecutarse una vez por cada ciclo para que no se bloquee
    self.main.read().run_instruction()
  }

  pub fn push_sub_thread(&self, thread: MultiRefHash<AsyncThread>) {
    self.sub_threads.write().unwrap().push_back(thread);
  }
  pub fn push_interrupt_thread(&self, thread: MultiRefHash<AsyncThread>) {
    self
      .interrupt_threads
      .write()
      .unwrap()
      .push_back(thread.clone());
  }
  fn poll_waiting_threads(&self) {
    if self.waiting_threads.read().unwrap().is_empty() {
      return; // No hay hilos en espera
    }
    // Clonamos los hilos en espera para no modificar la cola mientras iteramos
    let mut old_waiting_threads = self.waiting_threads.read().unwrap().clone();
    self.waiting_threads.write().unwrap().clear();
    while let Some(thread) = old_waiting_threads.pop_front() {
      if thread.read().is_waiting() {
        self.waiting_threads.write().unwrap().push_back(thread);
      } else {
        self.sub_threads.write().unwrap().push_back(thread);
      }
    }
  }
  fn run_interrupt_threads(&self) {
    if self.interrupt_threads.read().unwrap().is_empty() {
      return; // No hay hilos de interrupcion
    }
    // Clonamos los hilos de interrupcion para no modificar la cola mientras iteramos
    let mut old_interrupt_threads = self.interrupt_threads.read().unwrap().clone();
    self.interrupt_threads.write().unwrap().clear();
    while let Some(thread) = old_interrupt_threads.pop_front() {
      let response = thread.read().simple_run_instruction(false);
      if matches!(response, InterpretResult::Continue) {
        // Si el hilo continua, lo volvemos a meter en la cola de interrupcion
        self.interrupt_threads.write().unwrap().push_back(thread);
      }
    }
  }
}
