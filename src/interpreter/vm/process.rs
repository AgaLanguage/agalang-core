use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use crate::interpreter::{stack::InterpretResult, Thread};

use super::thread::{AsyncThread, ModuleThread};

#[derive(Debug)]
pub struct ProcessManager {
  main: Rc<RefCell<ModuleThread>>,
  // Se detecto que al no usar una referencia el ProcessManager sufria errores por
  //   varios prestamos mutables razon por la que se utiliza RefCell
  sub_threads: RefCell<VecDeque<Rc<RefCell<AsyncThread>>>>,
  waiting_threads: RefCell<VecDeque<Rc<RefCell<AsyncThread>>>>,
  interrupt_threads: RefCell<VecDeque<Rc<RefCell<AsyncThread>>>>,
}

impl ProcessManager {
  pub fn new(main: Rc<RefCell<ModuleThread>>) -> Self {
    Self {
      main,
      sub_threads: Default::default(),
      waiting_threads: Default::default(),
      interrupt_threads: Default::default(),
    }
  }
  pub fn as_value(&self) -> crate::compiler::Value {
    self.main.borrow().clone().as_value()
  }
  pub fn get_root_thread(&self) -> Rc<RefCell<Thread>> {
    self.main.borrow().get_async().borrow().get_thread()
  }
  pub fn run_instruction(&self) -> InterpretResult {
    // Ejecuta una instruccion de cada hilo de interrupcion, por ser prioritarios
    self.run_interrupt_threads();
    self.poll_waiting_threads();

    let first_item = self.sub_threads.borrow_mut().pop_front();
    if let Some(thread) = first_item {
      let response = thread.borrow().simple_run_instruction(true);
      if thread.borrow().is_waiting() {
        self.waiting_threads.borrow_mut().push_back(thread.clone());
      } else if matches!(response, InterpretResult::Continue) {
        self.sub_threads.borrow_mut().push_back(thread);
      }
    }

    // El hilo debe ejecutarse una vez por cada ciclo para que no se bloquee
    self.main.borrow().run_instruction()
  }

  pub fn push_sub_thread(&self, thread: Rc<RefCell<AsyncThread>>) {
    self.sub_threads.borrow_mut().push_back(thread);
  }
  fn poll_waiting_threads(&self) {
    if self.waiting_threads.borrow().is_empty() {
      return; // No hay hilos en espera
    }
    // Clonamos los hilos en espera para no modificar la cola mientras iteramos
    let mut old_waiting_threads = self.waiting_threads.borrow().clone();
    self.waiting_threads.borrow_mut().clear();
    while let Some(thread) = old_waiting_threads.pop_front() {
      if thread.borrow().is_waiting() {
        self.waiting_threads.borrow_mut().push_back(thread);
      } else {
        self.sub_threads.borrow_mut().push_back(thread);
      }
    }
  }
  fn run_interrupt_threads(&self) {
    if self.interrupt_threads.borrow().is_empty() {
      return; // No hay hilos de interrupcion
    }
    // Clonamos los hilos de interrupcion para no modificar la cola mientras iteramos
    let mut old_interrupt_threads = self.interrupt_threads.borrow().clone();
    self.interrupt_threads.borrow_mut().clear();
    while let Some(thread) = old_interrupt_threads.pop_front() {
      let response = thread.borrow().simple_run_instruction(true);
      if matches!(response, InterpretResult::Continue) {
        // Si el hilo continua, lo volvemos a meter en la cola de interrupcion
        self.interrupt_threads.borrow_mut().push_back(thread);
      }
    }
  }
}
