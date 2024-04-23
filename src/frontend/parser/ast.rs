pub enum NodeType {
    StringLiteral,
    NumberLiteral,
    String,
    Number,
    Program,
    VarDecl,
    Error,
}

impl Clone for NodeType {
    fn clone(&self) -> NodeType {
        match self {
            NodeType::StringLiteral => NodeType::StringLiteral,
            NodeType::NumberLiteral => NodeType::NumberLiteral,
            NodeType::String => NodeType::String,
            NodeType::Number => NodeType::Number,
            NodeType::Program => NodeType::Program,
            NodeType::VarDecl => NodeType::VarDecl,
            NodeType::Error => NodeType::Error,
        }
    }
}
impl Copy for NodeType {}

pub enum Node {
    Program(NodeProgram),
    StringLiteral(NodeStringLiteral),
    NumberLiteral(NodeNumberLiteral),
    VarDecl(NodeVarDecl),
    Error(NodeError),
}
impl std::fmt::Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Node::Program(node) => write!(f, "{}", node.to_string()),
      Node::StringLiteral(node) => write!(f, "{}", node.to_string()),
      Node::NumberLiteral(node) => write!(f, "{}", node.to_string()),
      Node::VarDecl(node) => write!(f, "{}", node.to_string()),
      Node::Error(node) => write!(f, "{}", node.to_string()),
    }
  }
}

pub trait DataNode {
    fn to_string(&self) -> String;
}
impl std::fmt::Display for dyn DataNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
pub struct NodeProgram {
    pub body: Vec<Node>,
    /* NodeType::Program */
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeProgram {
    fn to_string(&self) -> String {
        let str_body: Vec<String> = self.body.iter().map(|node| format!("  {}", node)).collect();
        format!("NodeProgram:\n{}", str_body.join("\n"))
    }
}
pub struct NodeStringLiteral {
    pub value: String,
    /* NodeType::StringLiteral */
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeStringLiteral {
    fn to_string(&self) -> String {
        format!("NodeStringLiteral: {}", self.value)
    }
}
pub struct NodeNumberLiteral {
    pub value: String,
    /* NodeType::NumberLiteral */
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeNumberLiteral {
    fn to_string(&self) -> String {
        format!("NodeNumberLiteral: {}", self.value)
    }
}
pub struct NodeString {
    pub value: String,
    /* NodeType::String */
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeString {
    fn to_string(&self) -> String {
        format!("NodeString: {}", self.value)
    }
}
pub struct NodeNumber {
    pub base: i8,
    pub value: String,
    /* NodeType::Number */
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeNumber {
    fn to_string(&self) -> String {
        format!("NodeNumber: {}", self.value)
    }
}
pub struct NodeVarDecl {
    pub name: String,
    pub value: Option<Box<Node>>,
    pub is_const: bool,
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeVarDecl {
    fn to_string(&self) -> String {
        let keyword = if self.is_const { "const" } else { "def" };
        match &self.value {
            Some(value) => format!("NodeVarDecl: {keyword} {} = {}", self.name, value.as_ref()),
            None => format!("NodeVarDecl: {keyword} {}", self.name),
        }
    }
}
pub struct NodeError {
    pub message: String,
    pub node_type: NodeType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeError {
    fn to_string(&self) -> String {
        format!("NodeError: {}", self.message)
    }
}