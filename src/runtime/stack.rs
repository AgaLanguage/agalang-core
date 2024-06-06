use crate::frontend::ast::Node;

#[derive(Clone)]
pub struct Stack {
  value: Box<Node>,
  previous: Option<Box<Stack>>,
}

impl Stack {
  pub fn next(&self, value: &Node) -> Stack {
    Stack {
      value: Box::new(value.clone()),
      previous: Some(Box::new(self.clone())),
    }
  }
  pub fn get_default() -> Stack {
    Stack {
      value: Box::new(Node::None),
      previous: None,
    }
  }
  pub fn iter(&self) -> Vec<&Node> {
    let mut stack = vec![self.get_value()];
    let mut current = self;
    while let Some(previous) = &current.previous {
      stack.push(previous.get_value());
      current = previous.as_ref();
    }
    stack
  }
  pub fn get_value(&self) -> &Node {
    self.value.as_ref()
  }
}

impl<'a> std::fmt::Display for Stack {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} -> {}", self.value, match &self.previous {
      Some(previous) => previous.to_string(),
      None => "None".to_string(),
    })
  }
}