use crate::{frontend::lexer::KeywordsType, internal};

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
    Class(NodeClass),
    Function(NodeFunction),
    If(NodeIf),
    Import(NodeImport),
    Export(NodeValue),
    // For(NodeFor),
    While(NodeWhile),
    DoWhile(NodeWhile),
    Try(NodeTry),
    Throw(NodeValue),

    // Expressions //
    UnaryFront(NodeUnary),
    UnaryBack(NodeUnary),
    Binary(NodeBinary),
    Member(NodeMember),
    Call(NodeCall),
    Return(NodeReturn),
    LoopEdit(NodeLoopEdit),
    Arrow(NodeArrow),
    Error(NodeError),
    PartialError(NodePartialError),
}
impl Node {
    pub fn is_error(&self) -> bool {
        match self {
            Node::Error(_) => true,
            _ => false,
        }
    }
    pub fn is_partial_error(&self) -> bool {
        match self {
            Node::PartialError(_) => true,
            _ => false,
        }
    }
    pub fn get_error(&self) -> Option<NodeError> {
        match self {
            Node::Error(node) => Some(node.clone()),
            Node::PartialError(node) => Some(node.error.clone()),
            _ => None,
        }
    }
    pub fn to_box(self) -> Box<Node> {
        Box::new(self)
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
            Node::Class(node) => node.column,
            Node::While(node) | Node::DoWhile(node) => node.column,
            Node::Try(node) => node.column,
            Node::Function(node) => node.column,
            Node::If(node) => node.column,
            Node::Import(node) => node.column,
            Node::Export(node) | Node::Throw(node) => node.column,
            Node::UnaryFront(node) | Node::UnaryBack(node) => node.column,
            Node::Binary(node) => node.column,
            Node::Member(node) => node.column,
            Node::Call(node) => node.column,
            Node::Return(node) => node.column,
            Node::LoopEdit(node) => node.column,
            Node::Arrow(node) => node.column,
            Node::Error(node) => node.column,
            Node::PartialError(node) => node.value.get_column(),
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
            Node::Class(node) => node.line,
            Node::While(node) | Node::DoWhile(node) => node.line,
            Node::Try(node) => node.line,
            Node::Function(node) => node.line,
            Node::If(node) => node.line,
            Node::Import(node) => node.line,
            Node::Export(node) | Node::Throw(node) => node.line,
            Node::UnaryFront(node) | Node::UnaryBack(node) => node.line,
            Node::Binary(node) => node.line,
            Node::Member(node) => node.line,
            Node::Call(node) => node.line,
            Node::Return(node) => node.line,
            Node::LoopEdit(node) => node.line,
            Node::Arrow(node) => node.line,
            Node::Error(node) => node.line,
            Node::PartialError(node) => node.value.get_line(),
        }
    }
    pub fn get_file(&self) -> String {
        let file: &str = match self {
            Node::Program(node) => &node.file,
            Node::String(node) => &node.file,
            Node::Number(node) => &node.file,
            Node::Object(node) => &node.file,
            Node::Array(node) => &node.file,
            Node::Identifier(node) => &node.file,
            Node::VarDecl(node) => &node.file,
            Node::Assignment(node) => &node.file,
            Node::Class(node) => &node.file,
            Node::While(node) | Node::DoWhile(node) => &node.file,
            Node::Try(node) => &node.file,
            Node::Function(node) => &node.file,
            Node::If(node) => &node.file,
            Node::Import(node) => &node.file,
            Node::Export(node) | Node::Throw(node) => &node.file,
            Node::UnaryFront(node) | Node::UnaryBack(node) => &node.file,
            Node::Binary(node) => &node.file,
            Node::Member(node) => &node.file,
            Node::Call(node) => &node.file,
            Node::Return(node) => &node.file,
            Node::LoopEdit(node) => &node.file,
            Node::Arrow(node) => &node.file,
            Node::Error(node) => &node.meta,
            Node::PartialError(node) => {
                let value: &str = &node.error.meta;
                let vec: Vec<_> = value.split("\0").collect();
                vec[0]
            }
        };
        return file.to_string();
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
            Node::Class(node) => Node::Class(node.clone()),
            Node::While(node) => Node::While(node.clone()),
            Node::DoWhile(node) => Node::DoWhile(node.clone()),
            Node::Try(node) => Node::Try(node.clone()),
            Node::Function(node) => Node::Function(node.clone()),
            Node::If(node) => Node::If(node.clone()),
            Node::Import(node) => Node::Import(node.clone()),
            Node::Export(node) => Node::Export(node.clone()),
            Node::Throw(node) => Node::Throw(node.clone()),
            Node::UnaryFront(node) => Node::UnaryFront(node.clone()),
            Node::UnaryBack(node) => Node::UnaryBack(node.clone()),
            Node::Binary(node) => Node::Binary(node.clone()),
            Node::Member(node) => Node::Member(node.clone()),
            Node::Call(node) => Node::Call(node.clone()),
            Node::Return(node) => Node::Return(node.clone()),
            Node::LoopEdit(node) => Node::LoopEdit(node.clone()),
            Node::Arrow(node) => Node::Arrow(node.clone()),
            Node::Error(node) => Node::Error(node.clone()),
            Node::PartialError(node) => Node::PartialError(node.clone()),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Node::Program(node) => {
                let data = node
                    .body
                    .iter()
                    .map(|n| format!("{n}"))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("NodeProgram:\n{}", data_format(data))
            }
            Node::String(node) => {
                let str_value: Vec<String> = node
                    .value
                    .iter()
                    .map(|data| match data {
                        StringData::Str(str) => format!("\"{}\"", str).replace("\n", "\\n"),
                        StringData::Id(id) => id.to_string(),
                    })
                    .collect();
                format!("NodeString: {}", str_value.join(" + "))
            }
            Node::Number(node) => format!("NodeNumber: {} en base {}", node.value, node.base),
            Node::Object(node) => {
                let str_properties: Vec<String> = node
                    .properties
                    .iter()
                    .map(|property| match property {
                        NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                        NodeProperty::Iterable(object) => {
                            format!("  ...({})", Node::Identifier(object.clone()))
                        }
                        NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                        NodeProperty::Indexable(value) => format!("  [{}]", value.to_string()),
                    })
                    .collect();
                format!(
                    "NodeObject: {{\n{}\n}}",
                    data_format(str_properties.join(",\n"))
                )
            }
            Node::Array(node) => {
                let str_elements: Vec<String> = node
                    .elements
                    .iter()
                    .map(|element| match element {
                        NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                        NodeProperty::Iterable(object) => {
                            format!("  ...({})", Node::Identifier(object.clone()))
                        }
                        NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                        NodeProperty::Indexable(value) => format!("  {}", value.to_string()),
                    })
                    .collect();
                format!("NodeArray: [\n{}\n]", data_format(str_elements.join(",\n")))
            }
            Node::Identifier(node) => format!("NodeIdentifier: {}", node.name),
            Node::VarDecl(node) => {
                let keyword = if node.is_const {
                    KeywordsType::Constante
                } else {
                    KeywordsType::Definir
                };
                let keyword = keyword.as_str();
                match &node.value {
                    Some(value) => format!(
                        "NodeVarDecl: {keyword} {}\n{}",
                        node.name,
                        data_format(value.to_string())
                    ),
                    None => format!("NodeVarDecl: {keyword} {}", node.name),
                }
            }
            Node::Assignment(node) => format!(
                "NodeAssignment: {}\n{}",
                node.identifier,
                data_format(node.value.to_string())
            ),
            Node::Class(node) => {
                let str_body: Vec<String> = node
                    .body
                    .iter()
                    .map(|p| {
                        let is_static = p.meta & 1 << 0 != 0;
                        let str_static = format!("static: {is_static}");
                        let is_const = p.meta & 1 << 1 != 0;
                        let str_const = format!("const: {is_const}");
                        let is_public = p.meta & 1 << 2 != 0;
                        let str_public = format!("public: {is_public}");
                        let str_info = format!("{str_static}\n{str_const}\n{str_public}");
                        let str_info = format!("{}:\n{}", p.name, data_format(str_info));
                        match &p.value {
                            Some(value) => {
                                format!("{str_info}\n{}", data_format(value.to_string()))
                            }
                            None => str_info,
                        }
                    })
                    .collect();
                format!(
                    "NodeClass: {}\n{}",
                    node.name,
                    data_format(str_body.join("\n"))
                )
            }
            Node::While(node) | Node::DoWhile(node) => {
                let str_body: Vec<String> = node.body.iter().map(|n| n.to_string()).collect();
                format!(
                    "NodeWhile:\n{}\n  <==>\n{}",
                    data_format(node.condition.to_string()),
                    data_format(str_body.join("\n"))
                )
            }
            Node::Try(node) => {
                let str_body: Vec<String> = node.body.iter().map(|n| n.to_string()).collect();
                let str_catch = format!(
                    "NodeTryCatch: {}:\n{}",
                    node.catch.0,
                    data_format(
                        node.catch
                            .1
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                );
                let str_finally = match &node.finally {
                    Some(finally) => format!(
                        "NodeTryFinally:\n{}",
                        data_format(
                            finally
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    ),
                    None => "No Finally".to_string(),
                };
                format!(
                    "NodeTry:\n{}\n  <==>\n{}\n  <==>\n{}",
                    data_format(str_body.join("\n")),
                    data_format(str_catch),
                    data_format(str_finally)
                )
            }
            Node::Function(node) => {
                let str_params = node.params.iter()
                .map(|arg| format!("{}", Node::Identifier(arg.clone())))
                .collect::<Vec<_>>()
                .join(", ");
                let str_body: Vec<_> = node.body.iter().map(|n| n.to_string()).collect();
                format!(
                    "NodeFunction: {} ({})\n{}",
                    node.name,
                    str_params,
                    data_format(str_body.join("\n"))
                )
            }
            Node::If(node) => {
                let str_body: Vec<String> = node.body.iter().map(|node| node.to_string()).collect();
                let str_else_body = match &node.else_body {
                    Some(else_body) => {
                        let str_else_body: Vec<String> =
                            else_body.iter().map(|node| node.to_string()).collect();
                        format!("\n  <==>\n{}", data_format(str_else_body.join("\n")))
                    }
                    None => "".to_string(),
                };
                format!(
                    "NodeIf:\n{}\n  <==>\n{}{}",
                    data_format(node.condition.to_string()),
                    data_format(str_body.join("\n")),
                    str_else_body
                )
            }
            Node::Import(node) => match &node.name {
                Some(name) => format!("NodeImport: {} como {}", node.path, name),
                None => format!("NodeImport: {}", node.path),
            },
            Node::Export(node) | Node::Throw(node) => {
                format!("NodeValue: \n{}", data_format(node.value.to_string()))
            }
            Node::UnaryFront(node) | Node::UnaryBack(node) => format!(
                "NodeUnary: \"{}\" para {{\n{}\n}}",
                node.operator,
                data_format(node.operand.to_string())
            ),
            Node::Binary(node) => format!(
                "NodeBinary:\n{}\n{}\n{}",
                data_format(node.left.to_string()),
                data_format(node.operator.clone()),
                data_format(node.right.to_string())
            ),
            Node::Member(node) => format!(
                "NodeMember:\n{}\n{}",
                data_format(node.object.to_string()),
                data_format(node.member.to_string())
            ),
            Node::Call(node) => {
                let str_arguments: Vec<String> = node
                    .arguments
                    .iter()
                    .map(|argument| format!("  {}", argument))
                    .collect();
                format!(
                    "NodeCall:\n{}\n  ({})",
                    data_format(node.callee.to_string()),
                    data_format(str_arguments.join("\n"))
                )
            }
            Node::Return(node) => match &node.value {
                Some(value) => format!("NodeReturn:\n{}", data_format(value.to_string())),
                None => "NodeReturn".to_string(),
            },
            Node::LoopEdit(node) => format!(
                "NodeLoopEdit: {}",
                match node.action {
                    NodeLoopEditType::Break => "break",
                    NodeLoopEditType::Continue => "continue",
                }
            ),
            Node::Arrow(node) => {
                let str_params = node.params.iter()
                .map(|arg| format!("{}", Node::Identifier(arg.clone())))
                .collect::<Vec<_>>()
                .join(", ");
                let str_body: Vec<_> = node.body.iter().map(|n| n.to_string()).collect();
                format!(
                    "NodeArrowFunction: ({})\n{}",
                    str_params,
                    data_format(str_body.join("\n"))
                )
            },
            Node::Error(node) => format!("NodeError: {}", node.message),
            Node::PartialError(node) => {
                let type_err = internal::ErrorNames::SyntaxError;
                let node_err = node.error.clone();
                let err = crate::frontend::node_error(&node_err);
                internal::show_warn(&type_err, err);
                format!(
                    "NodePartialError: \n  {}\n{}",
                    Node::Error(node.error.clone()),
                    data_format(node.value.to_string())
                )
            }
        };
        write!(f, "{}", str)
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
    Id(Box<Node>),
}
pub struct NodeString {
    pub value: Vec<StringData>,
    pub column: usize,
    pub line: usize,
    pub file: String,
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
    pub base: u8,
    pub value: String,
    pub column: usize,
    pub line: usize,
    pub file: String,
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
pub struct NodeIf {
    pub condition: Box<Node>,
    pub body: Vec<Node>,
    pub else_body: Option<Vec<Node>>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeIf {
    fn clone(&self) -> Self {
        NodeIf {
            condition: self.condition.clone(),
            body: self.body.iter().map(|node| node.clone()).collect(),
            else_body: match &self.else_body {
                Some(else_body) => Some(else_body.iter().map(|node| node.clone()).collect()),
                None => None,
            },
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub type NodeArguments = Vec<NodeIdentifier>;
pub struct NodeFunction {
    pub name: String,
    pub params: NodeArguments,
    pub body: Vec<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeFunction {
    fn clone(&self) -> Self {
        NodeFunction {
            name: self.name.clone(),
            params: self.params.iter().map(|param| param.clone()).collect(),
            body: self.body.iter().map(|node| node.clone()).collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeReturn {
    pub value: Option<Box<Node>>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeReturn {
    fn clone(&self) -> Self {
        NodeReturn {
            value: match &self.value {
                Some(value) => Some(value.clone()),
                None => None,
            },
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub enum NodeLoopEditType {
    Break,
    Continue,
}
impl Clone for NodeLoopEditType {
    fn clone(&self) -> Self {
        match self {
            NodeLoopEditType::Break => NodeLoopEditType::Break,
            NodeLoopEditType::Continue => NodeLoopEditType::Continue,
        }
    }
}
pub struct NodeLoopEdit {
    pub action: NodeLoopEditType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeLoopEdit {
    fn clone(&self) -> Self {
        NodeLoopEdit {
            action: self.action.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeTry {
    pub body: Vec<Node>,
    pub catch: (String, Vec<Node>),
    pub finally: Option<Vec<Node>>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeTry {
    fn clone(&self) -> Self {
        NodeTry {
            body: self.body.iter().map(|node| node.clone()).collect(),
            catch: (
                self.catch.0.clone(),
                self.catch.1.iter().map(|node| node.clone()).collect(),
            ),
            finally: match &self.finally {
                Some(finally) => Some(finally.iter().map(|node| node.clone()).collect()),
                None => None,
            },
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeClassProperty {
    pub name: String,
    pub value: Option<Box<Node>>,
    /** bits

    0: is_static
    1: is_const
    2: is_public */
    pub meta: u8,
}
pub struct NodeClass {
    pub name: String,
    pub body: Vec<NodeClassProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeClass {
    fn clone(&self) -> Self {
        NodeClass {
            name: self.name.clone(),
            body: self
                .body
                .iter()
                .map(|property| NodeClassProperty {
                    name: property.name.clone(),
                    value: match &property.value {
                        Some(value) => Some(value.clone()),
                        None => None,
                    },
                    meta: property.meta,
                })
                .collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeImport {
    pub path: String,
    pub name: Option<String>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeImport {
    fn clone(&self) -> Self {
        NodeImport {
            path: self.path.clone(),
            name: match &self.name {
                Some(name) => Some(name.clone()),
                None => None,
            },
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodeValue {
    pub value: Box<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeValue {
    fn clone(&self) -> Self {
        NodeValue {
            value: self.value.clone(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}
pub struct NodePartialError {
    pub error: NodeError,
    pub value: Box<Node>,
}
impl Clone for NodePartialError {
    fn clone(&self) -> Self {
        NodePartialError {
            error: self.error.clone(),
            value: self.value.clone(),
        }
    }
}
pub struct NodeArrow {
    pub params: NodeArguments,
    pub body: Vec<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
impl Clone for NodeArrow {
    fn clone(&self) -> Self {
        NodeArrow {
            params: self.params.iter().map(|param| param.clone()).collect(),
            body: self.body.iter().map(|node| node.clone()).collect(),
            column: self.column,
            line: self.line,
            file: self.file.clone(),
        }
    }
}