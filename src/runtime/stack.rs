use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use parser::ast::{BNode, Node};

use super::RefEnvironment;
#[derive(Clone, Debug, Default)]
pub struct RefStack(Rc<RefCell<Stack>>);
impl RefStack {
  pub fn crate_child(&self, in_class: bool, value: BNode) -> Self {
    Stack {
      env: self.env().crate_child(in_class),
      prev: Some(self.clone()),
      node: value,
    }.to_ref()
  }
  pub fn next(self, value: BNode) -> Self {
    self.0.borrow_mut().next(value);
    self
  }
  pub fn pop(self) -> Option<Self> {
    self.0.borrow_mut().pop()
  }
  pub fn current(&self) -> BNode {
    self.0.borrow().current()
  }
  pub fn env(&self) -> RefEnvironment {
    self.0.borrow().env()
  }
}
impl IntoIterator for RefStack {
  type Item = BNode;
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
  type Item = BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.clone().into_iter()
  }
}

#[derive(Clone, Debug, Default)]
pub struct Stack {
  node: BNode,
  env: RefEnvironment,
  prev: Option<RefStack>
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
    RefStack (Rc::new(RefCell::new(self)))
  }
  pub fn crate_child(&self, in_class: bool, value: BNode) -> Self {
    Self {
      env: self.env.crate_child(in_class),
      prev: Some(self.clone().to_ref()),
      node: value,
    }
  }
  pub fn next(&self, value: BNode) -> Self {
    Self {
      env: self.env.clone(),
      prev: Some(self.clone().to_ref()),
      node: value,
    }
  }
  pub fn pop(&self) -> Option<RefStack> {
    self.prev.clone()
  }
  pub fn current(&self) -> BNode {
    self.node.clone().to_box()
  }
  pub fn env(&self) -> RefEnvironment {
    self.env.clone()
  }
}
impl IntoIterator for Stack {
  type Item = BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.to_ref().into_iter()
  }
}
impl IntoIterator for &Stack {
  type Item = BNode;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.clone().to_ref().into_iter()
  }
}