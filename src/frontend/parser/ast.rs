pub enum Node {
    Program(NodeProgram),
    String(NodeString),
    Number(NodeNumber),
    Object(NodeObject),
    Identifier(NodeIdentifier),
    VarDecl(NodeVarDecl),
    Error(NodeError),
}
impl Node {
    pub fn is_error(&self) -> bool {
        match self {
            Node::Error(_) => true,
            _ => false,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Node::Program(node) => node.to_string(),
            Node::String(node) => node.to_string(),
            Node::Number(node) => node.to_string(),
            Node::Object(node) => node.to_string(),
            Node::Identifier(node) => node.to_string(),
            Node::VarDecl(node) => node.to_string(),
            Node::Error(node) => node.to_string(),
        }
    }
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

pub enum StringData {
    Str(String),
    Id(String),
}
pub struct NodeString {
    pub value: Vec<StringData>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeString {
    fn to_string(&self) -> String {
        let str_value: Vec<String> = self.value.iter().map(|data| match data {
            StringData::Str(str) => format!("\"{}\"", str),
            StringData::Id(id) => id.clone(),
        }).collect();
        format!("NodeString: {}", str_value.join(" + "))
    }
}
pub struct NodeNumber {
    pub base: i8,
    pub value: String,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeNumber {
    fn to_string(&self) -> String {
        format!("NodeNumber: {} en base {}", self.value, self.base)
    }
}
pub enum NodeProperty {
    Property(String, Node),
    Dynamic(Node, Node),
    Iterable(Node),
}
pub struct NodeObject {
    pub properties: Vec<NodeProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeObject {
    fn to_string(&self) -> String {
        let str_properties: Vec<String> = self.properties.iter().map(|property| match property {
            NodeProperty::Property(name, value) => format!("  {}: {}", name, value),
            NodeProperty::Iterable(object) => format!("  ...({})", object.to_string()),
            NodeProperty::Dynamic(name, value) => format!("  [{}]: {}", name, value),
        }).collect();
        format!("NodeObject: {{\n{}\n}}", str_properties.join(",\n"))
    }
}
pub struct NodeVarDecl {
    pub name: String,
    pub value: Option<Box<Node>>,
    pub is_const: bool,
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
pub struct NodeIdentifier {
    pub name: String,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeIdentifier {
    fn to_string(&self) -> String {
        format!("NodeIdentifier: {}", self.name)
    }
}
pub struct NodeError {
    pub message: String,
    pub column: usize,
    pub line: usize,
    pub meta: String,
}
impl DataNode for NodeError {
    fn to_string(&self) -> String {
        format!("NodeError: {}", self.message)
    }
}
