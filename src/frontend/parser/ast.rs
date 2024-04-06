pub enum NodeType {
  StringLiteral,
  NumberLiteral,
  Program,
  VarDecl,
}

pub trait Node {
  fn to_string(&self) -> String;
}
impl std::fmt::Display for dyn Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
}
}
pub struct NodeProgram {
  pub body: Vec<Box<dyn Node>>,
  /* NodeType::Program */
  pub node_type: NodeType,
  pub column: usize,
  pub line: usize,
  pub file: String,
}
impl Node for NodeProgram {
  fn to_string(&self) -> String {
    let str_body: Vec<String> = self.body.iter().map(|node| format!("  {}", node)).collect();
    format!("NodeProgram:\n{}", str_body.join("\n"))
  }
}
pub struct NodeStringLiteral {
  pub value: String,
  /* NodeType::String */
  pub node_type: NodeType,
  pub column: usize,
  pub line: usize,
  pub file: String,
}
impl Node for NodeStringLiteral {
  fn to_string(&self) -> String {
    format!("NodeString: {}", self.value)
  }
}
pub struct NodeNumberLiteral {
  pub value: f64,
  /* NodeType::Number */
  pub node_type: NodeType,
  pub column: usize,
  pub line: usize,
  pub file: String,
}
impl Node for NodeNumberLiteral {
  fn to_string(&self) -> String {
    format!("NodeNumber: {}", self.value)
  }
}
pub struct NodeVarDecl {
  pub name: String,
  pub value: Option<Box<dyn Node>>,
  pub is_const: bool,
  pub node_type: NodeType,
  pub column: usize,
  pub line: usize,
  pub file: String,
}
impl Node for NodeVarDecl {
  fn to_string(&self) -> String {
    let keyword = if self.is_const { "const" } else { "def" };
    match &self.value {
      Some(value) => format!("NodeVarDecl: {keyword} {} = {}", self.name, value),
      None => format!("NodeVarDecl: {keyword} {}", self.name)
    }
  }
}