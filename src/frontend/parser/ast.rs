use util::List; // is a Vec clonable

use crate::frontend::lexer::KeywordsType; // is a enum with keywords

pub type BNode = Box<Node>;

#[derive(Clone, PartialEq, Debug)]
pub enum Node {
    None,
    Program(NodeProgram),

    // Literals //
    String(NodeString),
    Number(NodeNumber),
    Object(NodeObject),
    Array(NodeArray),
    Identifier(NodeIdentifier),

    // Statements //
    VarDecl(NodeVarDecl),
    Name(NodeIdentifier),
    Assignment(NodeAssignment),
    Class(NodeClass),
    Function(NodeFunction),
    If(NodeIf),
    Import(NodeImport),
    Export(NodeValue),
    For(NodeFor),
    While(NodeWhile),
    DoWhile(NodeWhile),
    Try(NodeTry),
    Throw(NodeValue),
    Block(NodeBlock),

    // Expressions //
    UnaryFront(NodeUnary),
    UnaryBack(NodeUnary),
    Binary(NodeBinary),
    Member(NodeMember),
    Call(NodeCall),
    Return(NodeReturn),
    LoopEdit(NodeLoopEdit),
    Error(NodeError),
}
impl Node {
    pub fn is_identifier(&self) -> bool {
        match self {
            Node::Identifier(_) => true,
            _ => false,
        }
    }
    pub fn get_identifier(&self) -> Option<&NodeIdentifier> {
        match self {
            Node::Identifier(node) => Some(node),
            _ => None,
        }
    }
    pub fn is_error(&self) -> bool {
        match self {
            Node::Error(_) => true,
            _ => false,
        }
    }
    pub fn get_error(&self) -> Option<&NodeError> {
        match self {
            Node::Error(node) => Some(node),
            _ => None,
        }
    }
    pub fn to_box(self) -> BNode {
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
            Node::Name(node) => node.column,
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
            Node::For(node) => node.column,
            Node::Error(node) => node.column,
            Node::Block(_) | Node::None => 0,
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
            Node::Name(node) => node.line,
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
            Node::For(node) => node.line,
            Node::Error(node) => node.line,
            Node::Block(_) | Node::None => 0,
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
            Node::Name(node) => &node.file,
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
            Node::For(node) => &node.file,
            Node::Error(node) => &node.meta,
            Node::Block(_) | Node::None => "none",
        };
        return file.to_string();
    }
    pub fn get_type(&self) -> &str {
        match self {
            Node::Program(_) => "Programa",
            Node::String(_) => "Cadena",
            Node::Number(_) => "Numero",
            Node::Object(_) => "Objeto",
            Node::Array(_) => "Lista",
            Node::Identifier(_) => "Identificador",
            Node::VarDecl(_) => "Variable",
            Node::Name(_) => "Nombre",
            Node::Assignment(_) => "Asignacion",
            Node::Class(_) => "Clase",
            Node::While(_) => "Mientras",
            Node::DoWhile(_) => "Hacer",
            Node::Try(_) => "Intentar",
            Node::Function(_) => "Funcion",
            Node::If(_) => "Si",
            Node::Import(_) => "Importar",
            Node::Export(_) => "Exportar",
            Node::UnaryFront(_) => "Operador Unario",
            Node::UnaryBack(_) => "Operador Unario",
            Node::Binary(_) => "Operador Binario",
            Node::Member(_) => "Miembro",
            Node::Call(_) => "Llamada",
            Node::Return(_) => "Retorno",
            Node::LoopEdit(_) => "Editor de bucle",
            Node::For(_) => "Para",
            Node::Error(_) => "Error",
            Node::Block(_) => "Bloque",
            Node::None => "Nada",
            Node::Throw(_) => "Lanzar",
        }
    }
}


// TODO: delete this code


impl NodeBlock {
    pub fn join(&self, separator: &str) -> String {
        self.body.map(|node| format!("{}", node)).join(separator)
    }
}
impl std::fmt::Display for NodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str_body = self.body.map(|node| format!("{}", node)).join("\n");
        write!(f, "{}", data_format(str_body))
    }
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Node::Block(node) => node.body.to_string(),
            Node::Program(node) => format!("NodeProgram:\n{}", data_format(node.body.to_string())),
            Node::String(node) => {
                let str_value = node.value.map(|data| match data {
                    StringData::Str(str) => format!("\"{}\"", str).replace("\n", "\\n"),
                    StringData::Id(id) => id.to_string(),
                });
                format!("NodeString: {}", str_value)
            }
            Node::Number(node) => format!("NodeNumber: {} en base {}", node.value, node.base),
            Node::Object(node) => {
                let str_properties = node.properties.map(|property| match property {
                    NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                    NodeProperty::Iterable(object) => {
                        format!("  ...({})", Node::Identifier(object.clone()))
                    }
                    NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                    NodeProperty::Indexable(value) => format!("  [{}]", value.to_string()),
                });
                format!(
                    "NodeObject: {{\n{}\n}}",
                    data_format(str_properties.join(",\n"))
                )
            }
            Node::Array(node) => {
                let str_elements = node.elements.map(|element| match element {
                    NodeProperty::Property(name, value) => format!("  {}:\n  {}", name, value),
                    NodeProperty::Iterable(object) => {
                        format!("  ...({})", Node::Identifier(object.clone()))
                    }
                    NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
                    NodeProperty::Indexable(value) => format!("  {}", value.to_string()),
                });
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
            Node::Name(node) => format!("NodeName: {}", node.name),
            Node::Class(node) => {
                let str_body = node.body.map(|p| {
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
                });
                format!(
                    "NodeClass: {}\n{}",
                    node.name,
                    data_format(str_body.join("\n"))
                )
            }
            Node::While(node) | Node::DoWhile(node) => format!(
                "NodeWhile:\n{}\n  <==>\n{}",
                data_format(node.condition.to_string()),
                data_format(node.body.join("\n"))
            ),
            Node::Try(node) => {
                let str_catch = format!("NodeTryCatch: {}:\n{}", node.catch.0, node.catch.1);
                let str_finally = match &node.finally {
                    Some(finally) => format!("NodeTryFinally:\n{}", finally),
                    None => "No Finally".to_string(),
                };
                format!(
                    "NodeTry:\n  {}\n  <==>\n{}\n  <==>\n{}",
                    node.body,
                    data_format(str_catch),
                    data_format(str_finally)
                )
            }
            Node::Function(node) => {
                let str_params = node
                    .params
                    .map(|arg| format!("{}", Node::Identifier(arg.clone())))
                    .join(", ");
                format!(
                    "NodeFunction: {} ({})\n{}",
                    node.name,
                    str_params,
                    data_format(node.body.join("\n"))
                )
            }
            Node::If(node) => {
                let str_else_body = match &node.else_body {
                    Some(else_body) => format!("\n  <==>\n{}", data_format(else_body.join("\n"))),
                    None => "".to_string(),
                };
                format!(
                    "NodeIf:\n{}\n  <==>\n{}{}",
                    data_format(node.condition.to_string()),
                    data_format(node.body.join("\n")),
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
            Node::For(node) => format!(
                "NodeFor: \n{}\n{}\n{}\n  <==>\n{}",
                data_format(node.init.to_string()),
                data_format(node.condition.to_string()),
                data_format(node.update.to_string()),
                data_format(node.body.join("\n"))
            ),
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
                let str_arguments = node
                    .arguments
                    .map(|argument| format!("  {}", argument))
                    .join("\n");
                format!(
                    "NodeCall:\n{}\n  ({})",
                    data_format(node.callee.to_string()),
                    data_format(str_arguments)
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
            Node::Error(node) => format!("NodeError: {}", node.message),
            Node::None => "NodeNone".to_string(),
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
#[derive(Clone, PartialEq, Debug)]
pub struct NodeBlock {
    pub body: List<Node>,
    pub in_function: bool,
    pub in_loop: bool,
}
impl NodeBlock {
    pub fn len(&self) -> usize {
        self.body.len()
    }
    pub fn iter(&self) -> std::slice::Iter<Node> {
        self.body.iter()
    }
    pub fn to_node(self) -> Node {
        Node::Block(self)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeProgram {
    pub body: NodeBlock,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub enum StringData {
    Str(String),
    Id(String),
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeString {
    pub value: List<StringData>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeNumber {
    pub base: u8,
    pub value: String,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub enum NodeProperty {
    Property(String, Node),
    Dynamic(Node, Node),
    Iterable(NodeIdentifier),
    Indexable(Node),
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeObject {
    pub properties: List<NodeProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeArray {
    pub elements: List<NodeProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeVarDecl {
    pub name: String,
    pub value: Option<BNode>,
    pub is_const: bool,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeIdentifier {
    pub name: String,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeError {
    pub message: String,
    pub column: usize,
    pub line: usize,
    pub meta: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeUnary {
    pub operator: String,
    pub operand: BNode,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeBinary {
    pub operator: String,
    pub left: BNode,
    pub right: BNode,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeAssignment {
    pub identifier: BNode,
    pub value: BNode,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeMember {
    pub object: BNode,
    pub member: BNode,
    pub instance: bool,
    pub computed: bool,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeCall {
    pub callee: BNode,
    pub arguments: List<Node>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeWhile {
    pub condition: BNode,
    pub body: NodeBlock,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeIf {
    pub condition: BNode,
    pub body: NodeBlock,
    pub else_body: Option<NodeBlock>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeFunction {
    pub name: String,
    pub params: List<NodeIdentifier>,
    pub body: NodeBlock,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeReturn {
    pub value: Option<BNode>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub enum NodeLoopEditType {
    Break,
    Continue,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeLoopEdit {
    pub action: NodeLoopEditType,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeTry {
    pub body: NodeBlock,
    pub catch: (String, NodeBlock),
    pub finally: Option<NodeBlock>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeClassProperty {
    pub name: String,
    pub value: Option<BNode>,
    /** bits

    0: is_static
    1: is_const
    2: is_public */
    pub meta: u8,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeClass {
    pub name: String,
    pub body: List<NodeClassProperty>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeImport {
    pub path: String,
    pub name: Option<String>,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeValue {
    pub value: BNode,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeFor {
    pub init: BNode,
    pub condition: BNode,
    pub update: BNode,
    pub body: NodeBlock,
    pub column: usize,
    pub line: usize,
    pub file: String,
}
