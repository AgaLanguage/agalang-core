pub enum Node {
    Program(NodeProgram),

    // Literals //
    String(NodeString),
    Number(NodeNumber),
    Object(NodeObject),
    Array(NodeArray),
    Identifier(NodeIdentifier),

    // Statements //
    VarDecl(NodeVarDecl),
    Assignment(NodeAssignment),
    // Class(NodeClass),
    // Function(NodeFunction),
    // If(NodeIf),
    // Import(NodeImport),
    // Export(NodeExport),
    // For(NodeFor),
    While(NodeWhile),
    // DoWhile(NodeDoWhile),
    // Try(NodeTry),
    // Catch(NodeCatch),
    // Finally(NodeFinally),
    // Throw(NodeThrow),

    // Expressions //
    UnaryFront(NodeUnary),
    UnaryBack(NodeUnary),
    Binary(NodeBinary),
    Member(NodeMember),
    Call(NodeCall),
    // Arrow(NodeArrow),
    Error(NodeError),
}
impl Node {
    pub fn is_error(&self) -> bool {
        match self {
            Node::Error(_) => true,
            _ => false
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Node::Program(node) => node.to_string(),
            Node::String(node) => node.to_string(),
            Node::Number(node) => node.to_string(),
            Node::Object(node) => node.to_string(),
            Node::Array(node) => node.to_string(),
            Node::Identifier(node) => node.to_string(),
            Node::VarDecl(node) => node.to_string(),
            Node::Assignment(node) => node.to_string(),
            Node::While(node) => node.to_string(),
            Node::UnaryFront(node) => node.to_string(),
            Node::UnaryBack(node) => node.to_string(),
            Node::Binary(node) => node.to_string(),
            Node::Member(node) => node.to_string(),
            Node::Call(node) => node.to_string(),
            Node::Error(node) => node.to_string(),
        }
    }
    pub fn get_column(&self) -> usize {
        match self {
            Node::Program(node) => node.column,
            Node::String(node) => node.column,
            Node::Number(node) => node.column,
            Node::Object(node) => node.column,
            Node::Array(node) => node.column,
            Node::Identifier(node) => node.column,
            Node::VarDecl(node) => node.column,
            Node::Assignment(node) => node.column,
            Node::While(node) => node.column,
            Node::UnaryFront(node) => node.column,
            Node::UnaryBack(node) => node.column,
            Node::Binary(node) => node.column,
            Node::Member(node) => node.column,
            Node::Call(node) => node.column,
            Node::Error(node) => node.column,
        }
    }
    pub fn get_line(&self) -> usize {
        match self {
            Node::Program(node) => node.line,
            Node::String(node) => node.line,
            Node::Number(node) => node.line,
            Node::Object(node) => node.line,
            Node::Array(node) => node.line,
            Node::Identifier(node) => node.line,
            Node::VarDecl(node) => node.line,
            Node::Assignment(node) => node.line,
            Node::While(node) => node.line,
            Node::UnaryFront(node) => node.line,
            Node::UnaryBack(node) => node.line,
            Node::Binary(node) => node.line,
            Node::Member(node) => node.line,
            Node::Call(node) => node.line,
            Node::Error(node) => node.line,
        }
    }
    pub fn get_file(&self) -> String {
        match self {
            Node::Program(node) => node.file.clone(),
            Node::String(node) => node.file.clone(),
            Node::Number(node) => node.file.clone(),
            Node::Object(node) => node.file.clone(),
            Node::Array(node) => node.file.clone(),
            Node::Identifier(node) => node.file.clone(),
            Node::VarDecl(node) => node.file.clone(),
            Node::Assignment(node) => node.file.clone(),
            Node::While(node) => node.file.clone(),
            Node::UnaryFront(node) => node.file.clone(),
            Node::UnaryBack(node) => node.file.clone(),
            Node::Binary(node) => node.file.clone(),
            Node::Member(node) => node.file.clone(),
            Node::Call(node) => node.file.clone(),
            Node::Error(node) => node.meta.clone(),
        }
    }
}
impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Node::Program(node) => Node::Program(node.clone()),
            Node::String(node) => Node::String(node.clone()),
            Node::Number(node) => Node::Number(node.clone()),
            Node::Object(node) => Node::Object(node.clone()),
            Node::Array(node) => Node::Array(node.clone()),
            Node::Identifier(node) => Node::Identifier(node.clone()),
            Node::VarDecl(node) => Node::VarDecl(node.clone()),
            Node::Assignment(node) => Node::Assignment(node.clone()),
            Node::While(node) => Node::While(node.clone()),
            Node::UnaryFront(node) => Node::UnaryFront(node.clone()),
            Node::UnaryBack(node) => Node::UnaryBack(node.clone()),
            Node::Binary(node) => Node::Binary(node.clone()),
            Node::Member(node) => Node::Member(node.clone()),
            Node::Call(node) => Node::Call(node.clone()),
            Node::Error(node) => Node::Error(node.clone()),
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
fn data_format(data: String) -> String {
    data.split("\n")
        .map(|line| format!("  {}", line))
        .collect::<Vec<String>>()
        .join("\n")
}
pub struct NodeProgram {
    pub body: Vec<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeProgram {
    fn to_string(&self) -> String {
        let data = self
            .body
            .iter()
            .map(|node| format!("{}", node))
            .collect::<Vec<String>>()
            .join("\n");
        format!("NodeProgram:\n{}", data_format(data))
    }
}
impl Clone for NodeProgram {
    fn clone(&self) -> Self {
        NodeProgram {
            body: self.body.iter().map(|node| node.clone()).collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
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
        let str_value: Vec<String> = self
            .value
            .iter()
            .map(|data| match data {
                StringData::Str(str) => format!("\"{}\"", str).replace("\n", "\\n"),
                StringData::Id(id) => id.clone(),
            })
            .collect();
        format!("NodeString: {}", str_value.join(" + "))
    }
}
impl Clone for NodeString {
    fn clone(&self) -> Self {
        NodeString {
            value: self
                .value
                .iter()
                .map(|data| match data {
                    StringData::Str(str) => StringData::Str(str.clone()),
                    StringData::Id(id) => StringData::Id(id.clone()),
                })
                .collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
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
impl Clone for NodeNumber {
    fn clone(&self) -> Self {
        NodeNumber {
            base: self.base,
            value: self.value.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub enum NodeProperty {
    Property(String, Node),
    Dynamic(Node, Node),
    Iterable(NodeIdentifier),
    Indexable(Node),
}
pub struct NodeObject {
    pub properties: Vec<NodeProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeObject {
    fn to_string(&self) -> String {
        let str_properties: Vec<String> = self
            .properties
            .iter()
            .map(|property| match property {
                NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                NodeProperty::Iterable(object) => format!("  ...({})", object.to_string()),
                NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                NodeProperty::Indexable(value) => format!("  [{}]", value.to_string()),
            })
            .collect();
        format!(
            "NodeObject: {{\n{}\n}}",
            data_format(str_properties.join(",\n"))
        )
    }
}
impl Clone for NodeObject {
    fn clone(&self) -> Self {
        NodeObject {
            properties: self
                .properties
                .iter()
                .map(|property| match property {
                    NodeProperty::Property(name, value) => {
                        NodeProperty::Property(name.clone(), value.clone())
                    }
                    NodeProperty::Dynamic(name, value) => {
                        NodeProperty::Dynamic(name.clone(), value.clone())
                    }
                    NodeProperty::Iterable(object) => NodeProperty::Iterable(object.clone()),
                    NodeProperty::Indexable(value) => NodeProperty::Indexable(value.clone()),
                })
                .collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeArray {
    pub elements: Vec<NodeProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeArray {
    fn to_string(&self) -> String {
        let str_elements: Vec<String> = self
            .elements
            .iter()
            .map(|element| match element {
                NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                NodeProperty::Iterable(object) => format!("  ...({})", object.to_string()),
                NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                NodeProperty::Indexable(value) => format!("  {}", value.to_string()),
            })
            .collect();
        format!("NodeArray: [\n{}\n]", data_format(str_elements.join(",\n")))
    }
}
impl Clone for NodeArray {
    fn clone(&self) -> Self {
        NodeArray {
            elements: self
                .elements
                .iter()
                .map(|element| match element {
                    NodeProperty::Property(name, value) => {
                        NodeProperty::Property(name.clone(), value.clone())
                    }
                    NodeProperty::Dynamic(name, value) => {
                        NodeProperty::Dynamic(name.clone(), value.clone())
                    }
                    NodeProperty::Iterable(object) => NodeProperty::Iterable(object.clone()),
                    NodeProperty::Indexable(value) => NodeProperty::Indexable(value.clone()),
                })
                .collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
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
            Some(value) => format!(
                "NodeVarDecl: {keyword} {}\n{}",
                self.name,
                data_format(value.to_string())
            ),
            None => format!("NodeVarDecl: {keyword} {}", self.name),
        }
    }
}
impl Clone for NodeVarDecl {
    fn clone(&self) -> Self {
        NodeVarDecl {
            name: self.name.clone(),
            value: match &self.value {
                Some(value) => Some(value.clone()),
                None => None,
            },
            is_const: self.is_const,
            column: self.column,
            line: self.line,
            file: self.file.clone(),
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
impl Clone for NodeIdentifier {
    fn clone(&self) -> Self {
        NodeIdentifier {
            name: self.name.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
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
impl Clone for NodeError {
    fn clone(&self) -> Self {
        NodeError {
            message: self.message.clone(),
            column: self.column,
            line: self.line,
            meta: self.meta.clone(),
        }
    }
}
pub struct NodeUnary {
    pub operator: String,
    pub operand: Box<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeUnary {
    fn to_string(&self) -> String {
        format!("NodeUnary: \"{}\" para {{\n{}\n}}", self.operator, data_format(self.operand.to_string()))
    }
}
impl Clone for NodeUnary {
    fn clone(&self) -> Self {
        NodeUnary {
            operator: self.operator.clone(),
            operand: self.operand.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeBinary {
    pub operator: String,
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeBinary {
    fn to_string(&self) -> String {
        format!(
            "NodeBinary:\n{}\n{}\n{}",
            data_format(self.left.to_string()),
            data_format(self.operator.clone()),
            data_format(self.right.to_string())
        )
    }
}
impl Clone for NodeBinary {
    fn clone(&self) -> Self {
        NodeBinary {
            operator: self.operator.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeAssignment {
    pub identifier: Box<Node>,
    pub value: Box<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeAssignment {
    fn to_string(&self) -> String {
        format!(
            "NodeAssignment: {}\n{}",
            self.identifier,
            data_format(self.value.to_string())
        )
    }
}
impl Clone for NodeAssignment {
    fn clone(&self) -> Self {
        NodeAssignment {
            identifier: self.identifier.clone(),
            value: self.value.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}

pub struct NodeMember {
    pub object: Box<Node>,
    pub member: Box<Node>,
    pub computed: bool,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeMember {
    fn to_string(&self) -> String {
        format!(
            "NodeMember:\n{}\n{}",
            data_format(self.object.to_string()),
            data_format(self.member.to_string())
        )
    }
}
impl Clone for NodeMember {
    fn clone(&self) -> Self {
        NodeMember {
            object: self.object.clone(),
            member: self.member.clone(),
            computed: self.computed,
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}

pub struct NodeCall {
    pub callee: Box<Node>,
    pub arguments: Vec<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeCall {
    fn to_string(&self) -> String {
        let str_arguments: Vec<String> = self
            .arguments
            .iter()
            .map(|argument| format!("  {}", argument))
            .collect();
        format!(
            "NodeCall:\n{}\n  ({})",
            data_format(self.callee.to_string()),
            data_format(str_arguments.join("\n"))
        )
    }
}
impl Clone for NodeCall {
    fn clone(&self) -> Self {
        NodeCall {
            callee: self.callee.clone(),
            arguments: self.arguments.iter().map(|arg| arg.clone()).collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeWhile {
    pub condition: Box<Node>,
    pub body: Vec<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl DataNode for NodeWhile {
    fn to_string(&self) -> String {
        let str_body: Vec<String> = self
            .body
            .iter()
            .map(|node| node.to_string())
            .collect();
        format!(
            "NodeWhile:\n{}\n  <==>\n{}",
            data_format(self.condition.to_string()),
            data_format(str_body.join("\n"))
        )
    }
}

impl Clone for NodeWhile {
    fn clone(&self) -> Self {
        NodeWhile {
            condition: self.condition.clone(),
            body: self.body.iter().map(|node| node.clone()).collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}