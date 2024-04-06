pub mod ast;
pub fn produce_ast(source: String, is_function: bool, file_name: String) -> ast::NodeProgram {
  let tokens = super::tokenizer(source, file_name.clone());
  let mut program = ast::NodeProgram {
    body: Vec::new(),
    node_type: ast::NodeType::Program,
    column: 0,
    line: 0,
    file: file_name,
  };
  let mut i = 0;
  while i < tokens.len() {
    let (stmt, consumed) = parse_stmt(&tokens, i);
    if let Some(stmt) = stmt {
      program.body.push(stmt);
    }
    i += consumed;
  }
  program
}

fn parse_stmt(tokens: &Vec<util::Token<super::lexer::TokenType>>, i: usize, 
  is_function: bool,
  is_oop: bool,
  is_class_decl: bool,
  is_global_scope: bool) -> (Option<Box<dyn ast::Node>>, usize) {
  let token = &tokens[i];
  if token.token_type == super::lexer::TokenType::StringLiteral {
    let node = ast::NodeStringLiteral {
      value: token.value.clone(),
      node_type: ast::NodeType::StringLiteral,
      column: token.position.column,
      line: token.position.line,
      file: token.meta.clone(),
    };
    (Some(Box::new(node)), 1)
  } else
  if token.token_type == super::lexer::TokenType::NumberLiteral {
    let node = ast::NodeNumberLiteral {
      value: token.value.parse().unwrap(),
      node_type: ast::NodeType::NumberLiteral,
      column: token.position.column,
      line: token.position.line,
      file: token.meta.clone(),
    };
    (Some(Box::new(node)), 1)
  } else
  if token.token_type == super::lexer::TokenType::Keyword && (token.value == "def" || token.value == "const") {
    let (node, consumed) = parse_var_decl(tokens, i);
    (node, consumed)
  } else{
    (None, 1)
  }
}

fn parse_var_decl(tokens: &Vec<util::Token<super::lexer::TokenType>>, i: usize) -> (Option<Box<dyn ast::Node>>, usize) {
  let token = &tokens[i];
  let mut consumed = 1;
  let mut node = ast::NodeVarDecl {
    name: "".to_string(),
    value: None,
    is_const: token.value == "const",
    node_type: ast::NodeType::VarDecl,
    column: token.position.column,
    line: token.position.line,
    file: token.meta.clone(),
  };
  if tokens.len() > i + 1 {
    let next_token = &tokens[i + 1];
    if next_token.token_type == super::lexer::TokenType::Identifier {
      node.name = next_token.value.clone();
      consumed += 1;
    }
  }
  (Some(Box::new(node)), consumed)
}