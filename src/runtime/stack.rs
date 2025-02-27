use std::{cell::RefCell, rc::Rc};

use crate::parser;

use super::RefEnvironment;
#[derive(Clone, Debug)]
pub struct RefStack(Rc<RefCell<Stack>>);
impl RefStack {
  pub fn get_default() -> Self {
    Stack::new(RefEnvironment::get_default()).to_ref()
  }
  pub fn crate_child(&self, in_class: bool, value: parser::BNode) -> Self {
    Stack {
      env: self.env().crate_child(in_class),
      prev: Some(self.clone()),
      node: value,
    }
    .to_ref()
  }
  pub fn with_env(&self, env: RefEnvironment) -> Self {
    self.0.borrow().with_env(env).to_ref()
  }
  pub fn next(self, value: parser::BNode) -> Self {
    self.0.borrow_mut().next(value).to_ref()
  }
  pub fn pop(self) -> Option<Self> {
    self.0.borrow_mut().pop()
  }
  pub fn current(&self) -> parser::BNode {
    self.0.borrow().current()
  }
  pub fn env(&self) -> RefEnvironment {
    self.0.borrow().env()
  }
  pub fn get_global(&self) -> Self {
    Stack {
      env: self.env().get_global(),
      prev: Some(self.clone()),
      node: self.current(),
    }
    .to_ref()
  }
}
impl IntoIterator for RefStack {
  type Item = parser::BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    let mut stack = vec![];
    let mut current = Some(self);
    while let Some(mut current_val) = current {
      stack.push(current_val.current());
      current = current_val.pop();
    }
    stack.into_iter()
  }
}
impl IntoIterator for &RefStack {
  type Item = parser::BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.clone().into_iter()
  }
}

#[derive(Clone, Debug)]
pub struct Stack {
  node: parser::BNode,
  env: RefEnvironment,
  prev: Option<RefStack>,
}

impl Stack {
  fn new(env: RefEnvironment) -> Self {
    Self {
      env,
      prev: None,
      node: Default::default(),
    }
  }
  pub fn to_ref(self) -> RefStack {
    RefStack(Rc::new(RefCell::new(self)))
  }
  pub fn crate_child(&self, in_class: bool, value: parser::BNode) -> Self {
    Self {
      env: self.env.crate_child(in_class),
      prev: Some(self.clone().to_ref()),
      node: value,
    }
  }
  pub fn get_global(&self) -> Self {
    Self {
      env: self.env.get_global(),
      prev: Some(self.clone().to_ref()),
      node: self.node.clone(),
    }
  }
  pub fn with_env(&self, env: RefEnvironment) -> Self {
    Self {
      env,
      prev: Some(self.clone().to_ref()),
      node: self.node.clone(),
    }
  }
  pub fn next(&self, value: parser::BNode) -> Self {
    if (self.node == value) {
      return self.clone();
    }
    Self {
      env: self.env.clone(),
      prev: Some(self.clone().to_ref()),
      node: value,
    }
  }
  pub fn pop(&self) -> Option<RefStack> {
    self.prev.clone()
  }
  pub fn current(&self) -> parser::BNode {
    self.node.clone().to_box()
  }
  pub fn env(&self) -> RefEnvironment {
    self.env.clone()
  }
}
impl IntoIterator for Stack {
  type Item = parser::BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.to_ref().into_iter()
  }
}
impl IntoIterator for &Stack {
  type Item = parser::BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.clone().to_ref().into_iter()
  }
}
