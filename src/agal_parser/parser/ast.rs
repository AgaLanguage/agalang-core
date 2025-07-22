use crate::{
  agal_parser::{KeywordsType, TokenType},
  util,
};

impl<T> From<Node> for Result<Node, T> {
  fn from(node: Node) -> Result<Node, T> {
    Ok(node)
  }
}
impl<T, U> From<Node> for Result<Result<Node, T>, U> {
  fn from(node: Node) -> Result<Result<Node, T>, U> {
    Ok(Ok(node))
  }
}
pub type BNode = Box<Node>;
#[derive(Clone, PartialEq, Debug, Default, Eq, Hash)]
pub enum Node {
  #[default]
  None,
  Program(NodeProgram),

  // Literals //
  String(NodeString),
  Number(NodeNumber),
  Object(NodeObject),
  Array(NodeArray),
  Byte(NodeByte),
  Identifier(NodeIdentifier),

  // Statements //
  VarDecl(NodeVarDecl),
  VarDel(NodeIdentifier),
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
  Block(NodeBlock, bool),
  Await(NodeExpressionMedicator),
  Lazy(NodeExpressionMedicator),
  Console(NodeConsole),

  // Expressions //
  UnaryFront(NodeUnary),
  Binary(NodeBinary),
  Member(NodeMember),
  Call(NodeCall),
  Return(NodeReturn),
  LoopEdit(NodeLoopEdit),
}
impl Node {
  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }
  pub fn is_identifier(&self) -> bool {
    matches!(self, Self::Identifier(_))
  }
  pub fn get_identifier(&self) -> Option<&NodeIdentifier> {
    match self {
      Node::Identifier(node) => Some(node),
      _ => None,
    }
  }
  pub fn into_box(self) -> BNode {
    Box::new(self)
  }
  pub fn get_location(&self) -> util::Location {
    match self {
      Node::Await(node) | Node::Lazy(node) => node.location.clone(),
      Node::Byte(node) => node.location.clone(),
      Node::Program(node) => node.location.clone(),
      Node::String(node) => node.location.clone(),
      Node::Number(node) => node.location.clone(),
      Node::Object(node) => node.location.clone(),
      Node::Array(node) => node.location.clone(),
      Node::Identifier(node) | Node::Name(node) | Node::VarDel(node) => node.location.clone(),
      Node::VarDecl(node) => node.location.clone(),
      Node::Assignment(node) => node.location.clone(),
      Node::Class(node) => node.location.clone(),
      Node::While(node) | Node::DoWhile(node) => node.location.clone(),
      Node::Try(node) => node.location.clone(),
      Node::Function(node) => node.location.clone(),
      Node::If(node) => node.location.clone(),
      Node::Import(node) => node.location.clone(),
      Node::Export(node) | Node::Throw(node) => node.location.clone(),
      Node::UnaryFront(node) => node.location.clone(),
      Node::Binary(node) => node.location.clone(),
      Node::Member(node) => node.location.clone(),
      Node::Call(node) => node.location.clone(),
      Node::Return(node) => node.location.clone(),
      Node::LoopEdit(node) => node.location.clone(),
      Node::For(node) => node.location.clone(),
      Node::Block(node, _) => node.location.clone(),
      Node::Console(node) => match node {
        NodeConsole::Input { location, .. } => location,
        NodeConsole::Output { location, .. } => location,
        NodeConsole::Full { location, .. } => location,
      }
      .clone(),
      Node::None => util::Location {
        start: util::Position { line: 0, column: 0 },
        end: util::Position { line: 0, column: 0 },
        length: 0,
        file_name: "<Modulo Nativo>".to_string(),
      },
    }
  }
  pub fn get_file(&self) -> String {
    self.get_location().file_name
  }
  pub fn get_type(&self) -> &str {
    match self {
      Node::Lazy(_) => "Lazy",
      Node::Await(_) => "Await",
      Node::Byte(_) => "Byte",
      Node::Program(_) => "Programa",
      Node::String(_) => "Cadena",
      Node::Number(_) => "Numero",
      Node::Object(_) => "Objeto",
      Node::Array(_) => "Lista",
      Node::Identifier(_) => "Identificador",
      Node::VarDecl(_) => "Variable",
      Node::VarDel(_) => "VariableEliminada",
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
      Node::Binary(_) => "Operador Binario",
      Node::Member(_) => "Miembro",
      Node::Call(_) => "Llamada",
      Node::Return(_) => "Retorno",
      Node::LoopEdit(_) => "Editor de bucle",
      Node::For(_) => "Para",
      Node::Block(..) => "Bloque",
      Node::None => "Nada",
      Node::Console(_) => "Consola",
      Node::Throw(_) => "Lanzar",
    }
  }
}

impl NodeBlock {
  pub fn join(&self, separator: &str) -> String {
    self.body.map_ref(|node| node.to_string()).join(separator)
  }
}
impl std::fmt::Display for NodeBlock {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let str_body = self.body.map_ref(|node| node.to_string()).join("\n");
    write!(f, "{}", data_format(str_body))
  }
}
impl std::fmt::Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let str = match self {
      Node::Lazy(node) => format!("NodeLazy:\n  {}", node.expression),
      Node::Await(node) => format!("NodeAwait:\n  {}", node.expression),
      Node::Byte(node) => format!("NodeByte: {}", node.value),
      Node::Block(node, _) => node.body.to_string(),
      Node::Program(node) => format!("NodeProgram:\n{}", data_format(node.body.to_string())),
      Node::String(node) => {
        let str_value = node.value.map_ref(|data| match data {
          StringData::Str(str) => format!("\"{}\"", str).replace("\n", "\\n"),
          StringData::Id(id) => id.name.clone(),
        });
        format!("NodeString: {}", str_value)
      }
      Node::Number(node) => format!("NodeNumber: {} en base {}", node.value, node.base),
      Node::Object(node) => {
        let str_properties = node.properties.map_ref(|property| match property {
          NodeProperty::Property(id, value) => format!("  {}:\n  {}", id.name, value),
          NodeProperty::Iterable(object) => {
            format!("  ...({})", object)
          }
          NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
          NodeProperty::Indexable(value) => format!("  [{value}]"),
        });
        format!(
          "NodeObject: {{\n{}\n}}",
          data_format(str_properties.join(",\n"))
        )
      }
      Node::Array(node) => {
        let str_elements = node.elements.map_ref(|element| match element {
          NodeProperty::Property(id, value) => format!("  {}:\n  {}", id.name, value),
          NodeProperty::Iterable(object) => {
            format!("  ...({})", object)
          }
          NodeProperty::Dynamic(name, value) => format!("  [{}]:\n  {}", name, value),
          NodeProperty::Indexable(value) => format!("  {value}"),
        });
        format!("NodeArray: [\n{}\n]", data_format(str_elements.join(",\n")))
      }
      Node::Identifier(node) => format!("NodeIdentifier: {}", node.name),
      Node::VarDel(node) => format!("NodeVarDel: {}", node.name),
      Node::VarDecl(node) => {
        let keyword = if node.is_const {
          KeywordsType::Constant
        } else {
          KeywordsType::Define
        };
        let keyword = keyword.as_str();
        match &node.value {
          Some(value) => format!(
            "NodeVarDecl: {keyword} {}\n{}",
            node.name.name,
            data_format(value.to_string())
          ),
          None => format!("NodeVarDecl: {keyword} {}", node.name.name),
        }
      }
      Node::Assignment(node) => format!(
        "NodeAssignment: {}\n{}",
        node.identifier,
        data_format(node.value.to_string())
      ),
      Node::Name(node) => format!("NodeName: {}", node.name),
      Node::Class(node) => {
        let str_body = node.body.map_ref(|p| {
          let is_static = p.meta & 1 << 0 != 0;
          let str_static = format!("static: {is_static}");
          let is_const = p.meta & 1 << 1 != 0;
          let str_const = format!("const: {is_const}");
          let is_public = p.meta & 1 << 2 != 0;
          let str_public = format!("public: {is_public}");
          let str_info = format!("{str_static}\n{str_const}\n{str_public}");
          let str_info = format!("{}:\n{}", p.name.name, data_format(str_info));
          format!("{str_info}\n{}", data_format(p.value.to_string()))
        });
        format!(
          "NodeClass: {}\n{}",
          node.name.name,
          data_format(str_body.join("\n"))
        )
      }
      Node::While(node) | Node::DoWhile(node) => format!(
        "NodeWhile:\n{}\n  <==>\n{}",
        data_format(node.condition.to_string()),
        data_format(node.body.join("\n"))
      ),
      Node::Try(node) => {
        let str_catch = match &node.catch {
          Some(catch) => format!("NodeTryCatch: {}:\n{}", catch.0, catch.1),
          None => "No Catch".to_string(),
        };
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
          .map_ref(|arg| format!("{}", Node::Identifier(arg.clone())))
          .join(", ");
        format!(
          "NodeFunction: {} ({})\n{}",
          node.name.name,
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
        Some(identifier) => format!("NodeImport: {} como {}", node.path, identifier.name),
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
      Node::UnaryFront(node) => format!(
        "NodeUnary: \"{:?}\" para {{\n{}\n}}",
        node.operator,
        data_format(node.operand.to_string())
      ),
      Node::Binary(node) => format!(
        "NodeBinary:\n{}\n {:?}\n{}",
        data_format(node.left.to_string()),
        node.operator,
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
          .map_ref(|argument| format!("  {}", argument))
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
      Node::None => "NodeNone".to_string(),
      Node::Console(NodeConsole::Input { identifier, .. }) => {
        format!("NodeConsole: Input ({})", identifier.name)
      }
      Node::Console(NodeConsole::Output { value, .. }) => {
        format!("NodeConsole: Output\n{}", data_format(value.to_string()))
      }
      Node::Console(NodeConsole::Full {
        identifier, value, ..
      }) => format!(
        "NodeConsole: Output\n{}\nInput ({})",
        data_format(value.to_string()),
        identifier.name
      ),
    };
    write!(f, "{}", str)
  }
}
impl std::fmt::Display for NodeOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let str = match self {
      Self::None => "None",
      Self::LessThan => "<",
      Self::LessThanOrEqual => "<=",
      Self::BitMoveLeft => "<<",
      Self::BitMoveLeftEqual => "<<=",
      Self::GreaterThan => ">",
      Self::GreaterThanOrEqual => ">=",
      Self::BitMoveRight => ">>",
      Self::BitMoveRightEqual => ">>=",
      Self::Equal => "==",
      Self::Plus => "+",
      Self::PlusEqual => "+=",
      Self::Minus => "-",
      Self::MinusEqual => "-",
      Self::Multiply => "*",
      Self::MultiplyEqual => "*=",
      Self::Modulo => "%",
      Self::ModuloEqual => "%=",
      Self::Exponential => "^",
      Self::ExponentialEqual => "^=",
      Self::Division => "/",
      Self::DivisionEqual => "/=",
      Self::TruncDivision => "//",
      Self::TruncDivisionEqual => "//=",
      Self::QuestionMark => "?",
      Self::Nullish => "??",
      Self::NullishEqual => "??=",
      Self::BitAnd => "&",
      Self::BitAndEqual => "&=",
      Self::And => "&&",
      Self::AndEqual => "&&=",
      Self::BitOr => "|",
      Self::BitOrEqual => "|=",
      Self::Or => "||",
      Self::OrEqual => "||=",
      Self::Approximate => "~",
      Self::ApproximateEqual => "~=",
      Self::Not => "!",
      Self::NotEqual => "!=",
      Self::Assign => "=",
      Self::PipeLine => "|>",
      Self::At => "@",
    };
    write!(f, "{}", str)
  }
}
fn data_format(data: String) -> String {
  data
    .split("\n")
    .map(|line| format!("  {}", line))
    .collect::<Vec<String>>()
    .join("\n")
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum NodeConsole {
  Output {
    value: BNode,
    location: util::Location,
  },
  Input {
    location: util::Location,
    identifier: NodeIdentifier,
  },
  Full {
    value: BNode,
    location: util::Location,
    identifier: NodeIdentifier,
  },
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeBlock {
  pub body: util::List<Node>,
  pub in_function: bool,
  pub in_loop: bool,
  pub is_async: bool,
  pub location: util::Location,
}
impl NodeBlock {
  pub fn len(&self) -> usize {
    self.body.len()
  }
  pub fn is_empty(&self) -> bool {
    self.body.is_empty()
  }
  pub fn into_node(self) -> Node {
    let is_async = self.clone().is_async;
    Node::Block(self, is_async)
  }
  pub fn to_node(&self) -> Node {
    Node::Block(self.clone(), self.is_async)
  }
  pub fn iter(&self) -> std::slice::Iter<Node> {
    self.body.iter()
  }
}
impl IntoIterator for NodeBlock {
  type Item = Node;
  type IntoIter = std::vec::IntoIter<Node>;
  fn into_iter(self) -> Self::IntoIter {
    self.body.into_iter()
  }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeProgram {
  pub body: NodeBlock,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum StringData {
  Str(String),
  Id(NodeIdentifier),
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeString {
  pub value: util::List<StringData>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeNumber {
  pub base: u8,
  pub value: String,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeByte {
  pub value: u8,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum NodeProperty {
  Property(NodeIdentifier, Box<Node>),
  Dynamic(Box<Node>, Box<Node>),
  Iterable(Box<Node>),
  Indexable(Box<Node>),
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeObject {
  pub properties: util::List<NodeProperty>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeArray {
  pub elements: util::List<NodeProperty>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeVarDecl {
  pub name: NodeIdentifier,
  pub value: Option<BNode>,
  pub is_const: bool,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeIdentifier {
  pub name: String,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeError {
  pub message: String,
  pub location: util::Location,
}
impl NodeError {
  pub fn new(token: &util::Token<TokenType>, message: Option<String>) -> Self {
    Self {
      location: token.location.clone(),
      message: match message {
        Some(msg) => msg,
        None => format!("Error en {}", token.value),
      },
    }
  }
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeUnary {
  pub operator: NodeOperator,
  pub operand: BNode,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash, Copy)]
pub enum NodeOperator {
  /// This value is used when the operator is not defined
  None,
  /// <
  LessThan,
  /// <=
  LessThanOrEqual,
  /// <<
  BitMoveLeft,
  /// <<=
  BitMoveLeftEqual,
  /// >
  GreaterThan,
  /// >=
  GreaterThanOrEqual,
  /// >>
  BitMoveRight,
  /// >>=
  BitMoveRightEqual,
  /// +
  Plus,
  /// +=
  PlusEqual,
  /// -
  Minus,
  /// -=
  MinusEqual,
  /// *
  Multiply,
  /// *=
  MultiplyEqual,
  /// %
  Modulo,
  /// %=
  ModuloEqual,
  /// ^
  Exponential,
  /// ^=
  ExponentialEqual,
  /// /
  Division,
  /// /=
  DivisionEqual,
  /// //
  TruncDivision,
  /// //=
  TruncDivisionEqual,
  /// ?
  QuestionMark,
  /// ??
  Nullish,
  /// ??=
  NullishEqual,
  /// &
  BitAnd,
  /// &=
  BitAndEqual,
  /// &&
  And,
  /// &&=
  AndEqual,
  /// |
  BitOr,
  /// |=
  BitOrEqual,
  /// ||
  Or,
  /// ||=
  OrEqual,
  /// ~
  Approximate,
  /// ~=
  ApproximateEqual,
  /// !
  Not,
  /// !=
  NotEqual,
  /// =
  Assign,
  /// ==
  Equal,
  /// |>
  PipeLine,
  /// @
  At,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeBinary {
  pub operator: NodeOperator,
  pub left: BNode,
  pub right: BNode,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeAssignment {
  pub identifier: BNode,
  pub value: BNode,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeMember {
  pub object: BNode,
  pub member: BNode,
  pub instance: bool,
  pub computed: bool,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeCall {
  pub callee: BNode,
  pub arguments: util::List<Node>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeWhile {
  pub condition: BNode,
  pub body: NodeBlock,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeIf {
  pub condition: BNode,
  pub body: NodeBlock,
  pub else_body: Option<NodeBlock>,
  pub location: util::Location,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeFunction {
  pub is_async: bool,
  pub name: NodeIdentifier,
  pub params: util::List<NodeIdentifier>,
  pub body: NodeBlock,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeReturn {
  pub value: Option<BNode>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum NodeLoopEditType {
  Break,
  Continue,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeLoopEdit {
  pub action: NodeLoopEditType,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeTry {
  pub body: NodeBlock,
  pub catch: Option<(String, NodeBlock)>,
  pub finally: Option<NodeBlock>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeClassProperty {
  pub name: NodeIdentifier,
  pub value: BNode,
  /** bits
  1: is_static
  2: is_public */
  pub meta: u8,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeClass {
  pub name: NodeIdentifier,
  pub extend_of: Option<NodeIdentifier>,
  pub body: util::List<NodeClassProperty>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeImport {
  pub path: String,
  pub is_lazy: bool,
  pub name: Option<NodeIdentifier>,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeValue {
  pub value: BNode,
  pub location: util::Location,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeFor {
  pub init: BNode,
  pub condition: BNode,
  pub update: BNode,
  pub body: NodeBlock,
  pub location: util::Location,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NodeExpressionMedicator {
  pub expression: BNode,
  pub location: util::Location,
}
