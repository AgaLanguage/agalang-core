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
    self.main.read().clone().into_value()
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
    let mut waiting = self.waiting_threads.write().unwrap();
    let mut sub_threads = self.sub_threads.write().unwrap();

    let mut still_waiting = VecDeque::new();

    for thread in waiting.drain(..) {
      if thread.read().is_waiting() {
        still_waiting.push_back(thread);
      } else {
        sub_threads.push_back(thread);
      }
    }

    waiting.extend(still_waiting);
  }

  fn run_interrupt_threads(&self) {
    let mut interrupts = self.interrupt_threads.write().unwrap();
    let mut remaining = VecDeque::new();

    while let Some(thread) = interrupts.pop_front() {
      let response = thread.read().simple_run_instruction(false);
      if matches!(response, InterpretResult::Continue) {
        remaining.push_back(thread);
      }
    }

    interrupts.extend(remaining);
  }
}
