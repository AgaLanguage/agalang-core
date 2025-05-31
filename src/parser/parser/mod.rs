pub mod ast;
pub mod string;
pub use ast::*;

use crate::util::{self, OnError as _};

use super::KeywordsType;

const MISSING_TOKEN: &str = "\x1b[81mToken desaparecido\x1b[0m";

struct SemiToken {
  value: String,
  location: util::Location,
}
const COLOR: util::Color = util::Color::Cyan;

pub fn node_error(error: &ast::NodeError, source: &str) -> super::ErrorTypes {
  let line: usize;
  let column_node: usize;
  let message: &str;

  if error.message == MISSING_TOKEN {
    line = 0;
    column_node = 0;
    message = MISSING_TOKEN;
  } else {
    line = error.location.start.line + 1;
    column_node = error.location.start.column + 1;
    message = &error.message;
  }

  let binding = util::get_content(
    source,
    util::Position {
      line: error.location.start.line,
      column: 0,
    },
    error.location.end,
  )
  .unwrap();
  let lines = binding.lines().collect::<Vec<&str>>();
  let data_line = *lines.get(0).unwrap_or(&"");
  let node_value_len = error.location.end.column - error.location.start.column;
  let column = column_node + node_value_len;

  let str_line = line.to_string();
  let str_init = " ".repeat(str_line.len());

  let cyan_line = COLOR.apply("|");
  let cyan_arrow = COLOR.apply("-->");

  let pre_indicator = " ".repeat(error.location.start.column);
  let indicator = if node_value_len > 0 {
    format!("{}^", "-".repeat(node_value_len))
  } else {
    "^".to_string()
  };
  let lines = [
    format!("{}", message),
    format!(
      "{}{cyan_arrow} {}:{}:{}",
      str_init, error.location.file_name, line, column
    ),
    format!("{} {cyan_line}", str_init),
    format!("{} {cyan_line} {}", COLOR.apply(&str_line), data_line),
    format!(
      "{} {cyan_line} {pre_indicator}{}",
      str_init,
      COLOR.apply(&indicator)
    ),
    format!("{} {cyan_line}", str_init),
  ];
  let joined = lines.join("\n");
  super::ErrorTypes::StringError(joined)
}

pub struct Parser {
  source: String,
  tokens: Vec<util::Token<super::TokenType>>,
  index: usize,
  file_name: String,
}
impl Parser {
  pub fn new(source: &str, file_name: &str) -> Parser {
    let tokens = super::tokenizer(source, file_name);
    Parser {
      source: source.to_string(),
      tokens,
      index: 0,
      file_name: file_name.to_string(),
    }
  }
  fn is_eof(&mut self) -> bool {
    self.index >= self.tokens.len()
  }
  fn prev(&self) -> util::Token<super::TokenType> {
    let token = self.tokens.get(self.index - 1);
    if token.is_none() {
      return util::Token::<super::TokenType> {
        token_type: super::TokenType::Error,
        value: MISSING_TOKEN.to_string(),
        location: util::Location {
          start: util::Position { line: 0, column: 0 },
          end: util::Position { line: 0, column: 0 },
          length: 0,
          file_name: self.file_name.clone(),
        },
      };
    }
    let token = token.unwrap();
    util::Token::<super::TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location.clone(),
    }
  }
  fn at(&self) -> util::Token<super::TokenType> {
    let token = self.tokens.get(self.index);
    if token.is_none() {
      let location = self.prev().location.clone();
      return util::Token::<super::TokenType> {
        token_type: super::TokenType::Error,
        value: "Se esperaba un token".to_string(),
        location: location.clone(),
      };
    }
    let token = token.unwrap();
    util::Token::<super::TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location.clone(),
    }
  }
  fn eat(&mut self) -> util::Token<super::TokenType> {
    let token = self.at();
    self.index += 1;
    token
  }
  fn next(&self) -> util::Token<super::TokenType> {
    self.look(1)
  }
  fn look(&self, movement: usize) -> util::Token<super::TokenType> {
    let token = self.tokens.get(self.index + movement);
    if token.is_none() {
      let location = self.at().location.clone();
      return util::Token::<super::TokenType> {
        token_type: super::TokenType::Error,
        value: "Se esperaba un token".to_string(),
        location: location.clone(),
      };
    }
    let token = token.unwrap();
    util::Token::<super::TokenType> {
      token_type: token.token_type,
      value: token.value.clone(),
      location: token.location.clone(),
    }
  }
  fn match_token(&mut self, token_type: super::TokenType) -> bool {
    let result = self.at().token_type == token_type;
    if result {
      self.eat();
    }
    result
  }
  fn match_join_token(&mut self, token_type: super::TokenType) -> bool {
    let result = {
      let current = self.at();
      let prev = self.prev();
      current.token_type == token_type
        && current.location.start.line == prev.location.start.line
        && current.location.start.column == prev.location.end.column
    };
    if result {
      self.eat();
    }
    result
  }
  fn check_token(&mut self, token_type: super::TokenType) -> bool {
    self.at().token_type == token_type
  }
  fn check_in_tokens(&mut self, token_types: Vec<super::TokenType>) -> bool {
    for token_type in token_types {
      if self.check_token(token_type) {
        return true;
      }
    }
    false
  }
  fn expect(
    &mut self,
    token_type: super::TokenType,
    err: &str,
  ) -> Result<util::Token<super::TokenType>, NodeError> {
    let token = self.eat();
    if token.token_type != token_type {
      Err(NodeError {
        location: token.location.clone(),
        message: err.to_string(),
      })
    } else {
      Ok(util::Token::<super::TokenType> {
        token_type: token.token_type,
        value: token.value.clone(),
        location: token.location.clone(),
      })
    }
  }
  pub fn produce_ast(&mut self) -> Result<ast::Node, NodeError> {
    let body = self.parse_block(true, false, false, true, super::TokenType::EOF)?;
    let location = body.clone().location;
    ast::Node::Program(ast::NodeProgram { body, location }).into()
  }
  fn parse_stmt(
    &mut self,
    is_global_scope: bool,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      super::TokenType::EOF => {
        self.eat();
        Ok(ast::Node::default())
      }
      super::TokenType::Error => {
        return Err(ast::NodeError {
          message: token.value,
          location: token.location,
        });
      }
      super::TokenType::Keyword(key) => match key {
        super::KeywordsType::Define | super::KeywordsType::Constant => self.parse_var_decl(),
        super::KeywordsType::While
        | super::KeywordsType::Do
        | super::KeywordsType::If
        | super::KeywordsType::Function
        | super::KeywordsType::Try
        | super::KeywordsType::Class
        | super::KeywordsType::For
        | super::KeywordsType::Async => {
          self.parse_keyword_value(is_function, is_loop, is_async, false)
        }
        super::KeywordsType::Console => {
          let node = self.parse_keyword_value(is_function, is_loop, is_async, false);
          let semicolon = self.expect(
            super::TokenType::Punctuation(super::PunctuationType::SemiColon),
            "Se esperaba un punto y coma (stmt)",
          )?;
          if semicolon.token_type == super::TokenType::Error {
            Err(ast::NodeError {
              message: semicolon.value,
              location: semicolon.location,
            })
          } else {
            node
          }
        }
        super::KeywordsType::Return
        | super::KeywordsType::Continue
        | super::KeywordsType::Break => self.parse_simple_decl(is_function, is_loop),
        super::KeywordsType::Export => self.parse_export_decl(is_global_scope),
        super::KeywordsType::Import => self.parse_import_decl(is_global_scope),
        super::KeywordsType::Throw => self.parse_throw_decl(),
        super::KeywordsType::Await => {
          self.eat(); // await
          if !is_async {
            return Err(ast::NodeError {
              message: format!(
                "La palabra clave '{}' solo se puede utilizar en un contexto asíncrono",
                super::KeywordsType::Await.as_str()
              ),
              location: token.location,
            });
          }
          ast::Node::Await(ast::NodeExpressionMedicator {
            expression: self.parse_stmt_expr()?.to_box(),
            location: token.location,
          })
          .into()
        }
        super::KeywordsType::Delete => {
          self.eat();
          if self.match_token(super::TokenType::Identifier) {
            let name = self.prev().value;
            self.expect(
              super::TokenType::Punctuation(super::PunctuationType::SemiColon),
              &format!(
                "Se esperaba un punto y coma ({})",
                super::KeywordsType::Delete.to_string()
              ),
            )?;
            Ok(ast::Node::VarDel(ast::NodeIdentifier {
              name,
              location: token.location,
            }))
          } else {
            let at = self.at();
            Err(ast::NodeError {
              message: format!(
                "Se esperaba un identificador ({})",
                super::KeywordsType::Delete.to_string()
              ),
              location: at.location,
            })
          }
        }
        _ => {
          self.eat();
          Ok(ast::Node::default())
        }
      },
      _ => self.parse_stmt_expr(),
    }
  }
  fn parse_throw_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // lanzar
    let expr = self.parse_expr()?;
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      &format!(
        "Se esperaba un punto y coma ({})",
        super::KeywordsType::Throw.to_string()
      ),
    )?;
    ast::Node::Throw(ast::NodeValue {
      value: Box::new(expr),
      location: token.location,
    })
    .into()
  }
  fn parse_import_decl(&mut self, is_global_scope: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // importar
    let path = self.expect(
      super::TokenType::StringLiteral,
      "Se esperaba una ruta de archivo, debe usar una cadena literal con '",
    )?;
    let mut is_lazy = false;
    let mut name = None;
    if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::As) {
      self.eat();
      if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::Lazy) {
        self.eat();
        is_lazy = true;
      }
      let alias = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
      name = Some(alias.value.clone());
    }
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      &format!("Se esperaba un punto y coma ({})", path.value),
    )?;
    if !is_global_scope {
      return Err(ast::NodeError {
        message: "No se puede importar fuera del ámbito global".to_string(),
        location: token.location,
      });
    }
    ast::Node::Import(ast::NodeImport {
      path: path.value.clone(),
      name,
      is_lazy,
      location: token.location,
    })
    .into()
  }
  fn parse_export_decl(&mut self, is_global_scope: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // exportar
    let value = self.parse_export_value()?;
    if !is_global_scope {
      let error = ast::NodeError {
        message: "No se puede exportar fuera del ámbito global".to_string(),
        location: token.location,
      };
      let type_err = super::ErrorNames::SyntaxError;
      let err = node_error(&error, &self.source);
      let data = super::error_to_string(&type_err, err);
      super::print_warn(data);
      return Ok(value);
    }
    ast::Node::Export(ast::NodeValue {
      value: Box::new(value),
      location: token.location,
    })
    .into()
  }
  fn parse_export_value(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      super::TokenType::Keyword(super::KeywordsType::Define | super::KeywordsType::Constant) => {
        self.parse_var_decl()
      }
      super::TokenType::Keyword(super::KeywordsType::Function | super::KeywordsType::Class) => {
        self.parse_keyword_value(false, false, false, false)
      }
      super::TokenType::Keyword(super::KeywordsType::Name) => self.parse_name_decl(),
      _ => {
        self.eat();

        let message = if token.token_type == super::TokenType::Error {
          token.value
        } else {
          "Se esperaba un valor exportable".to_string()
        };
        Err(ast::NodeError {
          message,
          location: token.location,
        })
      }
    }
  }
  fn parse_name_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // nombre
    let name = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      &format!(
        "Se esperaba un punto y coma ({})",
        super::KeywordsType::Name.to_string()
      ),
    )?;
    ast::Node::Name(ast::NodeIdentifier {
      name: name.value.clone(),
      location: token.location,
    })
    .into()
  }
  fn parse_class_prop(
    &mut self,
    is_static: bool,
    is_public: bool,
  ) -> Result<ast::NodeClassProperty, ast::NodeError> {
    let is_async = if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::Async)
    {
      self.eat();
      true
    } else {
      false
    };
    let name = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
    let token = self.at();
    if token.token_type == super::TokenType::Error {
      self.eat();
      return Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
      });
    }
    let is_static_bit: u8 = if is_static { 1 } else { 0 };
    let is_public_bit: u8 = if is_public { 1 << 1 } else { 0 };
    let meta: u8 = is_static_bit | is_public_bit;
    if token.token_type == super::TokenType::Punctuation(super::PunctuationType::SemiColon) {
      return Ok(ast::NodeClassProperty {
        name: name.value.clone(),
        value: None,
        meta,
      });
    }
    let value: ast::Node = if token.token_type
      == super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen)
    {
      let params = self.parse_arguments_expr()?;
      let body = self.parse_block_expr(true, false, is_async)?;
      ast::Node::Function(ast::NodeFunction {
        is_async,
        name: name.value.clone(),
        params,
        body,
        location: token.location,
      })
    } else if token.token_type == super::TokenType::Operator(super::OperatorType::Equals) {
      self.eat();
      self.parse_expr()?
    } else {
      return Err(ast::NodeError {
        message: "Se esperaba un valor".to_string(),
        location: token.location,
      });
    };
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      &format!("Se esperaba un punto y coma ({})", name.value),
    )?;
    Ok(ast::NodeClassProperty {
      name: name.value.clone(),
      value: Some(value.to_box()),
      meta,
    })
  }
  fn parse_class_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // clase
    let name = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;

    let extend_of =
      if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::Extend) {
        self.eat();
        let class_node = self.parse_literal_expr("")?;
        if let ast::Node::Identifier(id) = class_node {
          Some(id)
        } else {
          return Err(ast::NodeError {
            message: "Se esperaba un identificador".to_string(),
            location: class_node.get_location(),
          });
        }
      } else {
        None
      };

    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::RegularBracketOpen),
      "Se esperaba un corchete de apertura",
    )?;
    let mut body = util::List::new();
    while !(self.is_eof()
      || self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::RegularBracketClose,
      )))
    {
      let (is_static, is_public) = self.get_modifier()?;

      let prop = self.parse_class_prop(is_static, is_public)?;
      body.push(prop);
    }
    ast::Node::Class(ast::NodeClass {
      name: name.value.clone(),
      extend_of,
      body,
      location: token.location,
    })
    .into()
  }
  fn get_modifier(&mut self) -> Result<(bool, bool), ast::NodeError> {
    let mut is_static = false;
    let mut is_public = false;
    while !self.is_eof() {
      let token = self.at();
      if token.token_type == super::TokenType::Error {
        self.eat();
        return Err(ast::NodeError {
          message: token.value.clone(),
          location: token.location,
        });
      }
      if self.match_token(super::TokenType::Keyword(super::KeywordsType::Static)) {
        if is_static {
          return Err(ast::NodeError {
            message: "Modificador duplicado".to_string(),
            location: token.location,
          });
        }
        is_static = true;
        continue;
      }
      if self.match_token(super::TokenType::Keyword(super::KeywordsType::Public)) {
        if is_public {
          return Err(ast::NodeError {
            message: "Modificador duplicado".to_string(),
            location: token.location,
          });
        }
        is_public = true;
        continue;
      }
      break;
    }
    Ok((is_static, is_public))
  }
  fn parse_simple_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); //super::TokenType::Keyword
                            // cont rom ret
    match token.token_type {
      super::TokenType::Keyword(super::KeywordsType::Return) => {
        if !is_function {
          return Err(ast::NodeError {
            message: "No se puede retornar fuera de una función".to_string(),
            location: token.location,
          });
        }
        let expr = self.parse_expr()?;
        self.expect(
          super::TokenType::Punctuation(super::PunctuationType::SemiColon),
          &format!(
            "Se esperaba un punto y coma ({})",
            super::KeywordsType::Return.to_string()
          ),
        )?;
        ast::Node::Return(ast::NodeReturn {
          value: Some(expr.to_box()),
          location: token.location,
        })
        .into()
      }
      super::TokenType::Keyword(super::KeywordsType::Break | super::KeywordsType::Continue) => {
        if !is_loop {
          return Err(ast::NodeError {
            message: "No se puede usar esta palabra clave fuera de un ciclo".to_string(),
            location: token.location,
          });
        }
        self.expect(
          super::TokenType::Punctuation(super::PunctuationType::SemiColon),
          "Se esperaba un punto y coma (Modificador de Bucle)",
        )?;
        let action = if token.value == KeywordsType::Continue.to_string() {
          ast::NodeLoopEditType::Continue
        } else {
          ast::NodeLoopEditType::Break
        };
        ast::Node::LoopEdit(ast::NodeLoopEdit {
          action,
          location: token.location,
        })
        .into()
      }
      super::TokenType::Error => Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
      }),
      _ => {
        return Err(ast::NodeError {
          message: "Token inesperado (simple)".to_string(),
          location: token.location,
        });
      }
    }
  }
  fn parse_keyword_value(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
    is_expr: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      super::TokenType::Keyword(super::KeywordsType::For) => {
        self.parse_for_decl(is_function, is_async)
      }
      super::TokenType::Keyword(super::KeywordsType::While) => {
        self.parse_while_decl(is_function, is_async)
      }
      super::TokenType::Keyword(super::KeywordsType::Do) => {
        self.parse_do_while_decl(is_function, is_async)
      }
      super::TokenType::Keyword(super::KeywordsType::If) => {
        self.parse_if_decl(is_function, is_loop, is_async)
      }
      super::TokenType::Keyword(super::KeywordsType::Function) => {
        self.parse_function_decl(false, is_expr)
      }
      super::TokenType::Keyword(super::KeywordsType::Async) => {
        self.eat();
        self.parse_function_decl(true, is_expr)
      }
      super::TokenType::Keyword(super::KeywordsType::Try) => {
        self.parse_try_decl(is_function, is_loop, is_async)
      }
      super::TokenType::Keyword(super::KeywordsType::Class) => self.parse_class_decl(),
      super::TokenType::Keyword(super::KeywordsType::Console) => {
        self.eat();
        let operator =
          if self.match_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
            if self.match_join_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
              ast::NodeOperator::BitMoveLeft
            } else {
              ast::NodeOperator::None
            }
          } else if self.match_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
            if self.match_join_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
              ast::NodeOperator::BitMoveRight
            } else {
              ast::NodeOperator::None
            }
          } else {
            ast::NodeOperator::None
          };
        if operator == ast::NodeOperator::BitMoveLeft {
          ast::Node::Console(ast::NodeConsole::Output {
            value: self.parse_expr()?.into(),
            location: token.location,
          })
          .into()
        } else if operator == ast::NodeOperator::BitMoveRight {
          let identifier =
            self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
          ast::Node::Console(ast::NodeConsole::Input {
            location: token.location,
            identifier: identifier.value,
          })
          .into()
        } else {
          Err(NodeError::new(
            &self.at(),
            Some("se esperaba un editor de bits".into()),
          ))
        }
      }
      super::TokenType::Keyword(super::KeywordsType::Lazy) => {
        self.eat();
        let expression = self.parse_expr()?.to_box();
        ast::Node::Lazy(ast::NodeExpressionMedicator {
          expression,
          location: token.location,
        })
        .into()
      }
      super::TokenType::Keyword(super::KeywordsType::Await) => {
        self.eat();
        let expression = self.parse_expr()?.to_box();
        ast::Node::Await(ast::NodeExpressionMedicator {
          expression,
          location: token.location,
        })
        .into()
      }
      super::TokenType::Error => {
        self.eat();
        Err(ast::NodeError {
          message: token.value.clone(),
          location: token.location,
        })
      }
      _ => Err(ast::NodeError {
        message: "Token inesperado (keyword)".to_string(),
        location: token.location,
      }),
    }
  }
  fn parse_for_decl(&mut self, is_function: bool, is_async: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // para
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen),
      "Se esperaba un paréntesis de apertura",
    )?;
    let init = self.parse_var_decl()?.to_box();
    let condition = self.parse_expr()?;
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      "Se esperaba un punto y coma (Para)",
    )?;
    let update = self.parse_expr()?;
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketClose),
      "Se esperaba un paréntesis de cierre",
    )?;
    let body = self.parse_block_expr(is_function, true, is_async)?;
    ast::Node::For(ast::NodeFor {
      init,
      condition: Box::new(condition),
      update: Box::new(update),
      body,
      location: token.location,
    })
    .into()
  }
  fn parse_try_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // intentar
    let body = self.parse_block_expr(is_function, is_loop, is_async)?;
    let catch = if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::Catch) {
      self.eat();
      self.expect(
        super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen),
        "Se esperaba un paréntesis de apertura",
      )?;
      let identifier = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
      self.expect(
        super::TokenType::Punctuation(super::PunctuationType::CircularBracketClose),
        "Se esperaba un paréntesis de cierre",
      )?;
      let block = self.parse_block_expr(is_function, is_loop, is_async)?;
      Some((identifier.value.clone(), block))
    } else {
      None
    };
    let finally = if self.at().token_type == super::TokenType::Keyword(super::KeywordsType::Finally)
    {
      self.eat();
      let block = self.parse_block_expr(is_function, is_loop, is_async)?;
      Some(block)
    } else {
      None
    };
    ast::Node::Try(ast::NodeTry {
      body,
      catch,
      finally,
      location: token.location,
    })
    .into()
  }
  fn parse_function_decl(&mut self, is_async: bool, is_expr: bool) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // fn
    let name = if is_expr {
      if self.check_token(super::TokenType::Identifier) {
        self.prev().value
      } else {
        "".into()
      }
    } else {
      self
        .expect(super::TokenType::Identifier, "Se esperaba un identificador")?
        .value
    };
    let params = self.parse_arguments_expr()?;
    let body = self.parse_block_expr(true, false, is_async)?;
    ast::Node::Function(ast::NodeFunction {
      is_async,
      name,
      params,
      body,
      location: token.location,
    })
    .into()
  }
  fn parse_arguments_expr(&mut self) -> Result<util::List<ast::NodeIdentifier>, ast::NodeError> {
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen),
      "Se esperaba un paréntesis de apertura",
    )?;
    let mut params = util::List::new();
    while !(self.is_eof()
      || self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::CircularBracketClose,
      )))
    {
      let param = if let super::TokenType::Operator(super::OperatorType::At) = self.at().token_type {
        self.eat(); // @
        let param = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
       ast::NodeIdentifier {
          name: format!("@{}", param.value),
          location: param.location,
        }
      }else {
        let param = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
        ast::NodeIdentifier {
          name: param.value.clone(),
          location: param.location,
        }
      };
      params.push(param);
      if self.match_token(super::TokenType::Punctuation(super::PunctuationType::Comma)) {
        continue;
      }
      if self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::CircularBracketClose,
      )) {
        break;
      }
      let comma = self.at();
      return Err(ast::NodeError {
        message: "Se esperaba una coma (args)".to_string(),
        location: comma.location,
      });
    }
    Ok(params)
  }
  fn parse_if_decl(
    &mut self,
    is_function: bool,
    is_loop: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // si
    let condition = self.parse_expr()?;
    let body = self.parse_block_expr(is_function, is_loop, is_async)?;
    let else_token = self.at(); // ent
    let else_body = if else_token.token_type == super::TokenType::Keyword(super::KeywordsType::Else)
    {
      self.eat();
      let else_block = self.parse_block_expr(is_function, is_loop, is_async)?;
      if else_block.len() == 0 {
        None
      } else {
        Some(else_block)
      }
    } else {
      None
    };
    ast::Node::If(ast::NodeIf {
      condition: condition.to_box(),
      body,
      else_body,
      location: token.location,
    })
    .into()
  }
  fn parse_do_while_decl(
    &mut self,
    is_function: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // hacer
    let body = self.parse_block_expr(is_function, true, is_async)?;
    self.expect(
      super::TokenType::Keyword(super::KeywordsType::While),
      &format!(
        "Se esperaba la palabra clave '{}'",
        super::KeywordsType::While.as_str()
      ),
    )?;
    let condition = self.parse_expr()?.to_box();
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      &format!(
        "Se esperaba un punto y coma ({})",
        super::KeywordsType::Do.to_string()
      ),
    )?;
    ast::Node::DoWhile(ast::NodeWhile {
      condition,
      body,
      location: token.location,
    })
    .into()
  }
  fn parse_while_decl(
    &mut self,
    is_function: bool,
    is_async: bool,
  ) -> Result<ast::Node, NodeError> {
    let token = self.eat(); // mien
    let condition = self.parse_expr()?;
    let body = self.parse_block_expr(is_function, true, is_async)?;
    ast::Node::While(ast::NodeWhile {
      condition: condition.to_box(),
      body,
      location: token.location,
    })
    .into()
  }
  fn parse_block_expr(
    &mut self,
    in_function: bool,
    in_loop: bool,
    is_async: bool,
  ) -> Result<NodeBlock, ast::NodeError> {
    let open_brace = self.at();
    if open_brace.token_type == super::TokenType::Error {
      return Err(ast::NodeError {
        message: "Se esperaba un bloque".to_string(),
        location: open_brace.location,
      });
    }
    if !self.match_token(super::TokenType::Punctuation(
      super::PunctuationType::RegularBracketOpen,
    )) {
      let expr = self.parse_stmt(false, in_function, in_loop, is_async)?;
      let mut body = util::List::new();
      let location = expr.get_location();
      body.push(expr);
      return Ok(ast::NodeBlock {
        body,
        in_function,
        in_loop,
        is_async,
        location,
      });
    }
    self.parse_block(
      false,
      in_function,
      in_loop,
      is_async,
      super::TokenType::Punctuation(super::PunctuationType::RegularBracketClose),
    )
  }
  fn parse_block(
    &mut self,
    is_global_scope: bool,
    in_function: bool,
    in_loop: bool,
    is_async: bool,
    stop_with: super::TokenType,
  ) -> Result<NodeBlock, ast::NodeError> {
    let mut functions = Vec::new();
    let mut code = Vec::new();
    loop {
      let is_eof = self.is_eof();
      let is_stop = self.match_token(stop_with);
      if is_eof || is_stop {
        break;
      }
      let stmt = self.parse_stmt(is_global_scope, in_function, in_loop, is_async)?;
      match stmt {
        ast::Node::Function(_) => functions.push(stmt),
        ast::Node::Export(ref export) => match export.value.as_ref() {
          ast::Node::Function(_) => functions.push(stmt.clone()),
          _ => code.push(stmt),
        },
        _ => code.push(stmt),
      }
    }
    let mut body = util::List::new();
    body.append_vec(&mut functions);
    body.append_vec(&mut code);
    Ok(ast::NodeBlock {
      body,
      in_function,
      in_loop,
      is_async,
      location: util::Location {
        start: util::Position { line: 0, column: 0 },
        end: util::Position { line: 0, column: 0 },
        length: 0,
        file_name: self.file_name.clone(),
      },
    })
  }
  fn parse_var_decl(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    let is_const = token.value == "const";
    let mut semi_token = SemiToken {
      value: token.value,
      location: token.location.clone(),
    };

    let identifier = self.expect(super::TokenType::Identifier, "Se esperaba un identificador")?;
    if semi_token.location.start.line == identifier.location.start.line {
      semi_token.value += " "
        .repeat(identifier.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.line = identifier.location.start.line;
    semi_token.location.start.column = identifier.location.start.column;
    if identifier.token_type == super::TokenType::Error {
      return Err(ast::NodeError {
        message: identifier.value,
        location: semi_token.location,
      });
    }
    semi_token.value += identifier.value.as_str();

    let equals_semicolon = self.eat();
    if semi_token.location.start.line == equals_semicolon.location.start.line {
      semi_token.value += " "
        .repeat(equals_semicolon.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.line = equals_semicolon.location.start.line;
    semi_token.location.start.column = equals_semicolon.location.start.column;
    if equals_semicolon.token_type
      == super::TokenType::Punctuation(super::PunctuationType::SemiColon)
    {
      return ast::Node::VarDecl(ast::NodeVarDecl {
        name: identifier.value.clone(),
        value: None,
        is_const,
        location: identifier.location,
      })
      .into();
    }
    if equals_semicolon.token_type != super::TokenType::Operator(super::OperatorType::Equals) {
      return Err(ast::NodeError {
        message: format!("Se esperaba un punto y coma (variable e)"),
        location: semi_token.location,
      });
    }
    semi_token.value += equals_semicolon.value.as_str();

    let value = self.parse_expr()?;
    if semi_token.location.start.line == value.get_location().start.line {
      semi_token.value += " "
        .repeat(value.get_location().start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.column = value.get_location().start.column;
    let semicolon = self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      "Se esperaba un punto y coma (variable v)",
    )?;
    if semi_token.location.start.line == semicolon.location.start.line {
      semi_token.value += " "
        .repeat(semicolon.location.start.column - semi_token.location.start.column)
        .as_str();
    } else {
      semi_token.value = "".to_string();
    };
    semi_token.location.start.column += 1;
    ast::Node::VarDecl(ast::NodeVarDecl {
      name: identifier.value.clone(),
      value: Some(value.to_box()),
      is_const,
      location: token.location,
    })
    .into()
  }
  fn parse_stmt_expr(&mut self) -> Result<ast::Node, NodeError> {
    let node = self.parse_expr()?;
    self.expect(
      super::TokenType::Punctuation(super::PunctuationType::SemiColon),
      "Se esperaba un punto y coma (expr)",
    )?;
    Ok(node)
  }
  fn parse_expr(&mut self) -> Result<ast::Node, NodeError> {
    let left = self.parse_pipeline_expr()?;
    self.parse_complex_expr(left)
  }
  fn parse_pipeline_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_logic_expr()?;
    loop {
      let token = self.at();
      if token.token_type != super::TokenType::Operator(super::OperatorType::Or) {
        return Ok(left);
      }
      if self.next().token_type != super::TokenType::Operator(super::OperatorType::GreaterThan) {
        return Ok(left);
      }
      if !self.match_token(super::TokenType::Operator(super::OperatorType::Or)) {
        return Ok(left);
      };
      if !self.match_join_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
        return Ok(left);
      }
      let right = self.parse_logic_expr()?;
      let mut arguments = util::List::new();
      arguments.push(left.clone());
      left = ast::Node::Call(ast::NodeCall {
        callee: right.to_box(),
        arguments,
        location: left.get_location(),
      })
    }
  }
  fn parse_logic_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_equals_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(
        super::OperatorType::Or | super::OperatorType::And | super::OperatorType::QuestionMark,
      ) = token.token_type
      {
        let next = if token.token_type == self.next().token_type {
          self.look(2).token_type
        } else {
          self.next().token_type
        };
        if let super::TokenType::Operator(
          super::OperatorType::Equals | super::OperatorType::GreaterThan | super::OperatorType::Or,
        ) = next
        {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = if self.match_token(super::TokenType::Operator(super::OperatorType::Or)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Or)) {
          ast::NodeOperator::Or
        } else {
          return Ok(left);
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::And)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::And)) {
          ast::NodeOperator::And
        } else {
          return Ok(left);
        }
      } else if self.match_token(super::TokenType::Operator(
        super::OperatorType::QuestionMark,
      )) {
        if self.match_join_token(super::TokenType::Operator(
          super::OperatorType::QuestionMark,
        )) {
          ast::NodeOperator::Nullish
        } else {
          return Ok(left);
        }
      } else {
        return Ok(left);
      };
      let right = self.parse_equals_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    }
  }
  fn parse_equals_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_comparison_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(super::OperatorType::Not | super::OperatorType::Equals) =
        token.token_type
      {
        if self.next().token_type != super::TokenType::Operator(super::OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = if self.match_token(super::TokenType::Operator(super::OperatorType::Not)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::NotEqual
        } else {
          return Ok(left);
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Equals)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::Equal
        } else {
          return Ok(left);
        }
      } else {
        return Ok(left);
      };
      let right = self.parse_comparison_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    }
  }
  fn parse_comparison_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_move_bits_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(
        super::OperatorType::GreaterThan | super::OperatorType::LessThan,
      ) = token.token_type
      {
        if self.next().token_type != super::TokenType::Operator(super::OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator =
        if self.match_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::GreaterThanOrEqual
          } else {
            ast::NodeOperator::GreaterThan
          }
        } else if self.match_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::LessThanOrEqual
          } else {
            ast::NodeOperator::LessThan
          }
        } else {
          return Ok(left);
        };
      let right = self.parse_move_bits_expr()?;
      if operator == ast::NodeOperator::GreaterThanOrEqual {
        let greater_than = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::GreaterThan,
          left: left.clone().to_box(),
          right: right.clone().to_box(),
          location: left.get_location(),
        });
        let equal = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::Equal,
          left: left.clone().to_box(),
          right: right.clone().to_box(),
          location: left.get_location(),
        });
        left = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::Or,
          left: greater_than.to_box(),
          right: equal.to_box(),
          location: left.get_location(),
        });
      } else if operator == ast::NodeOperator::LessThanOrEqual {
        let less_than = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::LessThan,
          left: left.clone().to_box(),
          right: right.clone().to_box(),
          location: left.get_location(),
        });
        let equal = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::Equal,
          left: left.clone().to_box(),
          right: right.clone().to_box(),
          location: left.get_location(),
        });
        left = ast::Node::Binary(ast::NodeBinary {
          operator: ast::NodeOperator::Or,
          left: less_than.to_box(),
          right: equal.to_box(),
          location: left.get_location(),
        });
      } else {
        left = ast::Node::Binary(ast::NodeBinary {
          operator,
          left: left.clone().to_box(),
          right: right.to_box(),
          location: left.get_location(),
        })
      }
    }
  }
  fn parse_move_bits_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_bit_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(
        super::OperatorType::GreaterThan | super::OperatorType::LessThan,
      ) = token.token_type
      {
        let next = if token.token_type == self.next().token_type {
          self.look(2).token_type
        } else {
          return Ok(left);
        };
        if next == super::TokenType::Operator(super::OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator =
        if self.match_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
            ast::NodeOperator::BitMoveRight
          } else {
            return Ok(left);
          }
        } else if self.match_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
            ast::NodeOperator::BitMoveLeft
          } else {
            return Ok(left);
          }
        } else {
          return Ok(left);
        };
      let right = self.parse_bit_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    }
  }
  fn parse_bit_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_math_lineal_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(super::OperatorType::And | super::OperatorType::Or) =
        token.token_type
      {
        if let super::TokenType::Operator(
          super::OperatorType::Equals | super::OperatorType::GreaterThan | super::OperatorType::Or,
        ) = self.next().token_type
        {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = if self.match_token(super::TokenType::Operator(super::OperatorType::And)) {
        ast::NodeOperator::BitAnd
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Or)) {
        ast::NodeOperator::BitOr
      } else {
        return Ok(left);
      };
      let right = self.parse_math_lineal_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    }
  }
  fn parse_math_lineal_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_math_multiplicative_expr()?;
    loop {
      let token = self.at();
      if let super::TokenType::Operator(super::OperatorType::Plus | super::OperatorType::Minus) =
        token.token_type
      {
        if self.next().token_type == super::TokenType::Operator(super::OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = if self.match_token(super::TokenType::Operator(super::OperatorType::Plus)) {
        ast::NodeOperator::Plus
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Minus)) {
        ast::NodeOperator::Minus
      } else {
        return Ok(left);
      };
      let right = self.parse_math_multiplicative_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      });
    }
  }
  fn parse_math_multiplicative_expr(&mut self) -> Result<ast::Node, NodeError> {
    let mut left = self.parse_math_exponential_expr()?;
    loop {
      let token = self.at();

      if let super::TokenType::Operator(
        super::OperatorType::Star | super::OperatorType::Division | super::OperatorType::Modulo,
      ) = token.token_type
      {
        let next = if token.token_type == self.next().token_type {
          self.look(2).token_type
        } else {
          self.next().token_type
        };
        if next == super::TokenType::Operator(super::OperatorType::Equals) {
          return Ok(left);
        }
      } else {
        return Ok(left);
      }
      let operator = if self.match_token(super::TokenType::Operator(super::OperatorType::Star)) {
        ast::NodeOperator::Multiply
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Division)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Division)) {
          ast::NodeOperator::TruncDivision
        } else {
          ast::NodeOperator::Division
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Modulo)) {
        ast::NodeOperator::Modulo
      } else {
        return Ok(left);
      };
      let right = self.parse_math_exponential_expr()?;
      left = ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    }
  }
  fn parse_math_exponential_expr(&mut self) -> Result<ast::Node, NodeError> {
    let left = self.parse_simple_expr("Token inesperado (exponencial iz)")?;
    let token = self.at();
    if token.token_type != super::TokenType::Operator(super::OperatorType::Exponential) {
      return left.into();
    }
    if self.next().token_type == super::TokenType::Operator(super::OperatorType::Equals) {
      return left.into();
    }
    if !self.match_token(super::TokenType::Operator(super::OperatorType::Exponential)) {
      return left.into();
    };
    let right = self
      .parse_simple_expr("Token inesperado (exponencial de)")?
      .to_box();
    ast::Node::Binary(ast::NodeBinary {
      operator: ast::NodeOperator::Exponential,
      left: left.clone().to_box(),
      right,
      location: left.get_location(),
    })
    .into()
  }
  fn parse_simple_expr(&mut self, message: &str) -> Result<ast::Node, NodeError> {
    let value = self.parse_literal_expr(message)?;
        if self.check_in_tokens(vec![
      super::TokenType::Punctuation(super::PunctuationType::Dot),
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen),
      super::TokenType::Punctuation(super::PunctuationType::QuadrateBracketOpen),
      super::TokenType::Punctuation(super::PunctuationType::DoubleDot),
    ]) {
      self.parse_call_member_expr(value)?.into()
    } else {
      value.into()
    }
  }
  fn parse_complex_expr(&mut self, left: ast::Node) -> Result<ast::Node, NodeError> {
    let left: ast::Node = if self.check_in_tokens(vec![
      super::TokenType::Punctuation(super::PunctuationType::Dot),
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen),
      super::TokenType::Punctuation(super::PunctuationType::QuadrateBracketOpen),
      super::TokenType::Punctuation(super::PunctuationType::DoubleDot),
    ]) {
      self.parse_call_member_expr(left)?.into()
    } else {
      left.into()
    };
    let token = self.at();
    if token.token_type == super::TokenType::Error {
      return Err(ast::NodeError {
        message: token.value.clone(),
        location: token.location,
      });
    }
    let sintaxis_operator =
      if self.match_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::LessThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::BitMoveLeftEqual
          } else {
            ast::NodeOperator::BitMoveLeft
          }
        } else if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::LessThanOrEqual
        } else {
          ast::NodeOperator::LessThan
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::BitMoveRightEqual
          } else {
            ast::NodeOperator::BitMoveRight
          }
        } else if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::GreaterThanOrEqual
        } else {
          ast::NodeOperator::GreaterThan
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Plus)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::PlusEqual
        } else {
          ast::NodeOperator::Plus
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Minus)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::MinusEqual
        } else {
          ast::NodeOperator::Minus
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Modulo)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::ModuloEqual
        } else {
          ast::NodeOperator::Modulo
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Exponential)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::ExponentialEqual
        } else {
          ast::NodeOperator::Exponential
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Division)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Division)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::TruncDivisionEqual
          } else {
            ast::NodeOperator::TruncDivision
          }
        } else if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::DivisionEqual
        } else {
          ast::NodeOperator::Division
        }
      } else if self.match_token(super::TokenType::Operator(
        super::OperatorType::QuestionMark,
      )) {
        if self.match_join_token(super::TokenType::Operator(
          super::OperatorType::QuestionMark,
        )) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::NullishEqual
          } else {
            ast::NodeOperator::Nullish
          }
        } else {
          ast::NodeOperator::QuestionMark
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::And)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::And)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::AndEqual
          } else {
            ast::NodeOperator::And
          }
        } else if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::BitAndEqual
        } else {
          ast::NodeOperator::BitAnd
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Or)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Or)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
            ast::NodeOperator::OrEqual
          } else {
            ast::NodeOperator::Or
          }
        } else if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::BitOrEqual
        } else {
          ast::NodeOperator::BitOr
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Approximate)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::ApproximateEqual
        } else {
          ast::NodeOperator::Approximate
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Not)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::NotEqual
        } else {
          ast::NodeOperator::Not
        }
      } else if self.match_token(super::TokenType::Operator(super::OperatorType::Equals)) {
        if self.match_join_token(super::TokenType::Operator(super::OperatorType::Equals)) {
          ast::NodeOperator::Equal
        } else {
          ast::NodeOperator::Assign
        }
      } else {
        ast::NodeOperator::None
      };
    if sintaxis_operator == ast::NodeOperator::BitMoveRight
      && self.match_token(super::TokenType::Keyword(super::KeywordsType::Console))
    {
      let (value, identifier) =
        if self.match_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
          if self.match_join_token(super::TokenType::Operator(super::OperatorType::GreaterThan)) {
            if self.match_token(super::TokenType::Identifier) {
              (left.into(), Some(self.prev().value))
            } else {
              let token = self.eat();

              return Err(ast::NodeError {
                message: "Falta el nombre del identificador".to_string(),
                location: token.location,
              });
            }
          } else {
            let token = self.eat();

            return Err(ast::NodeError {
              message: "Falta el operador >>".to_string(),
              location: token.location,
            });
          }
        } else {
          (left.into(), None)
        };
      return match identifier {
        Some(identifier) => ast::Node::Console(ast::NodeConsole::Full {
          location: token.location,
          identifier,
          value,
        }),
        None => ast::Node::Console(ast::NodeConsole::Output {
          value,
          location: token.location,
        }),
      }
      .into();
    }
    let (operator, is_assignment) = match sintaxis_operator {
      ast::NodeOperator::None => return left.into(),
      ast::NodeOperator::PlusEqual => (ast::NodeOperator::Plus, true),
      ast::NodeOperator::MinusEqual => (ast::NodeOperator::Minus, true),
      ast::NodeOperator::MultiplyEqual => (ast::NodeOperator::Multiply, true),
      ast::NodeOperator::DivisionEqual => (ast::NodeOperator::Division, true),
      ast::NodeOperator::ModuloEqual => (ast::NodeOperator::Modulo, true),
      ast::NodeOperator::ExponentialEqual => (ast::NodeOperator::Exponential, true),
      ast::NodeOperator::TruncDivisionEqual => (ast::NodeOperator::TruncDivision, true),
      ast::NodeOperator::BitAndEqual => (ast::NodeOperator::BitAnd, true),
      ast::NodeOperator::BitOrEqual => (ast::NodeOperator::BitOr, true),
      ast::NodeOperator::BitMoveLeftEqual => (ast::NodeOperator::BitMoveLeft, true),
      ast::NodeOperator::BitMoveRightEqual => (ast::NodeOperator::BitMoveRight, true),
      ast::NodeOperator::NullishEqual => (ast::NodeOperator::Nullish, true),
      ast::NodeOperator::ApproximateEqual => (ast::NodeOperator::Approximate, true),
      ast::NodeOperator::AndEqual => (ast::NodeOperator::And, true),
      ast::NodeOperator::OrEqual => (ast::NodeOperator::Or, true),
      ast::NodeOperator::Assign => (ast::NodeOperator::None, true),
      x => (x, false),
    };

    let right = if operator == ast::NodeOperator::None && is_assignment {
      self.parse_expr()?
    } else if operator == ast::NodeOperator::None {
      return left.into();
    } else if operator == ast::NodeOperator::LessThanOrEqual
      || operator == ast::NodeOperator::GreaterThanOrEqual
    {
      let right = self.parse_expr()?;
      let value_than = ast::Node::Binary(ast::NodeBinary {
        operator: if operator == ast::NodeOperator::LessThanOrEqual {
          NodeOperator::LessThan
        } else {
          NodeOperator::GreaterThan
        },
        left: left.clone().to_box(),
        right: right.clone().to_box(),
        location: left.get_location(),
      });
      let equal = ast::Node::Binary(ast::NodeBinary {
        operator: NodeOperator::Equal,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      });
      ast::Node::Binary(ast::NodeBinary {
        operator: NodeOperator::Or,
        left: value_than.clone().to_box(),
        right: equal.to_box(),
        location: value_than.get_location(),
      })
    } else {
      let right = self.parse_expr()?;
      ast::Node::Binary(ast::NodeBinary {
        operator,
        left: left.clone().to_box(),
        right: right.to_box(),
        location: left.get_location(),
      })
    };
    if is_assignment {
      ast::Node::Assignment(ast::NodeAssignment {
        identifier: left.clone().to_box(),
        value: right.to_box(),
        location: left.get_location(),
      })
      .into()
    } else {
      right.into()
    }
  }
  fn parse_call_member_expr(&mut self, object: ast::Node) -> Result<ast::Node, NodeError> {
    let member = self.parse_member_expr(object)?;
    if self.check_token(super::TokenType::Punctuation(
      super::PunctuationType::CircularBracketOpen,
    )) {
      return self.parse_call_expr(member);
    }
    member.into()
  }
  fn parse_call_expr(&mut self, callee: ast::Node) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    let mut args = util::List::new();
    while !(self.is_eof()
      || self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::CircularBracketClose,
      )))
    {
      let arg = self.parse_expr()?;
      args.push(arg);
      if self.check_token(super::TokenType::Punctuation(
        super::PunctuationType::CircularBracketClose,
      )) || self.match_token(super::TokenType::Punctuation(super::PunctuationType::Comma))
      {
        continue;
      }
      let comma = self.at();
      return Err(ast::NodeError {
        message: "Se esperaba una coma (args l)".to_string(),
        location: comma.location,
      });
    }
    let call_expr = ast::Node::Call(ast::NodeCall {
      callee: callee.to_box(),
      arguments: args,
      location: token.location,
    });
    self.parse_complex_expr(call_expr)
  }
  fn parse_member_expr(&mut self, object: ast::Node) -> Result<ast::Node, NodeError> {
    let mut value = object;
    loop {
      let object = self.match_token(super::TokenType::Punctuation(super::PunctuationType::Dot));
      let instance = if object {
        false
      } else if self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::DoubleDot,
      )) {
        if self.match_join_token(super::TokenType::Punctuation(
          super::PunctuationType::DoubleDot,
        )) {
          true
        } else {
          return Err(NodeError {
            message: "Se esperaban dos punto '::'".to_string(),
            location: self.at().location,
          });
        }
      } else {
        false
      };
      let computed = self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::QuadrateBracketOpen,
      ));
      if !(object || computed || instance) {
        break;
      }
      let property = if computed {
        self.parse_expr()
      } else {
        self.parse_literal_member_expr()
      }?;
      if computed {
        self.expect(
          super::TokenType::Punctuation(super::PunctuationType::QuadrateBracketClose),
          "Se esperaba un corchete cuadrado de cierre (pme)",
        )?;
      }
      value = ast::Node::Member(ast::NodeMember {
        object: value.clone().to_box(),
        member: property.to_box(),
        computed,
        instance,
        location: value.get_location(),
      });
    }
    value.into()
  }
  fn parse_literal_member_expr(&mut self) -> Result<ast::Node, NodeError> {
    let token = self.eat();
    match token.token_type {
      super::TokenType::Identifier | super::TokenType::Keyword(_) => {
        ast::Node::Identifier(ast::NodeIdentifier {
          location: token.location,

          name: token.value,
        })
        .into()
      }
      _ => Err(ast::NodeError {
        location: token.location,
        message: "Se esperaba un identificador valido".to_string(),
      }),
    }
  }
  fn parse_literal_expr(&mut self, message: &str) -> Result<ast::Node, NodeError> {
    let token = self.at();
    match token.token_type {
      super::TokenType::Identifier => ast::Node::Identifier(ast::NodeIdentifier {
        name: self.eat().value,
        location: token.location,
      })
      .into(),
      super::TokenType::NumberLiteral => ast::Node::Number(ast::NodeNumber {
        base: 10,
        value: self.eat().value,
        location: token.location,
      })
      .into(),
      super::TokenType::Number => {
        self.eat();
        let data = token.value.split("n").collect::<Vec<_>>()[1];
        let base_value = data.split("|").collect::<Vec<_>>();
        let base = base_value[0].parse::<u8>().unwrap();
        let value = base_value[1].to_string();
        ast::Node::Number(ast::NodeNumber {
          base,
          value,
          location: token.location,
        })
        .into()
      }
      super::TokenType::Byte => ast::Node::Byte(ast::NodeByte {
        value: u8::from_str_radix(&self.eat().value, 2)
          .on_error(|_| self.prev())
          .on_error(|token| NodeError {
            message: if message != "" {
              message.to_string()
            } else {
              token.value
            },
            location: token.location,
          })?,
        location: token.location,
      })
      .into(),
      super::TokenType::StringLiteral => ast::Node::String(ast::NodeString {
        value: util::List::from_vec(vec![ast::StringData::Str(self.eat().value)]),
        location: token.location,
      })
      .into(),
      super::TokenType::String => {
        self.eat();

        let node = string::complex_string(token)?;
        ast::Node::String(node).into()
      }
      super::TokenType::Punctuation(super::PunctuationType::RegularBracketOpen) => {
        self.parse_object_expr()
      }
      super::TokenType::Punctuation(super::PunctuationType::CircularBracketOpen) => {
        self.eat();
        let expr = self.parse_expr()?;
        self.expect(
          super::TokenType::Punctuation(super::PunctuationType::CircularBracketClose),
          "Se esperaba un paréntesis de cierre",
        )?;
        Ok(expr)
      }
      super::TokenType::Punctuation(super::PunctuationType::QuadrateBracketOpen) => {
        self.parse_array_expr()
      }
      super::TokenType::Operator(
        super::OperatorType::Minus
        | super::OperatorType::Plus
        | super::OperatorType::Approximate
        | super::OperatorType::Not
        | super::OperatorType::And
        | super::OperatorType::QuestionMark
        |super::OperatorType::At,
      ) => {
        self.eat();
        let operand = self.parse_literal_expr(message)?.to_box();
        let operator = if let super::TokenType::Operator(op) = token.token_type {
          op
        } else {
          return Err(NodeError {
            message: if message != "" {
              message.to_string()
            } else {
              token.value
            },
            location: token.location,
          });
        };
        let operator = if operator == super::OperatorType::Minus {
          ast::NodeOperator::Minus
        } else if operator == super::OperatorType::Plus {
          ast::NodeOperator::Plus
        } else if operator == super::OperatorType::Approximate {
          ast::NodeOperator::Approximate
        } else if operator == super::OperatorType::Not {
          ast::NodeOperator::Not
        } else if operator == super::OperatorType::And {
          ast::NodeOperator::BitAnd
        } else if operator == super::OperatorType::At {
          ast::NodeOperator::At
        }else {
          ast::NodeOperator::QuestionMark
        };
        ast::Node::UnaryFront(ast::NodeUnary {
          operator,
          operand,
          location: token.location,
        })
        .into()
      }
      super::TokenType::Keyword(
        super::KeywordsType::While
        | super::KeywordsType::Do
        | super::KeywordsType::If
        | super::KeywordsType::Function
        | super::KeywordsType::Try
        | super::KeywordsType::Async
        | super::KeywordsType::Console
        | super::KeywordsType::Await,
      ) => self.parse_keyword_value(false, false, false, true),
      _ => Err(NodeError {
        message: if message != "" {
          message.to_string()
        } else {
          token.value
        },
        location: token.location,
      }),
    }
  }
  fn parse_object_expr(&mut self) -> Result<ast::Node, NodeError> {
    let open_brace = self.eat();
    let mut properties = util::List::new();

    while !(self.is_eof()
      || self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::RegularBracketClose,
      )))
    {
      let property = self.parse_object_property()?;
      properties.push(property);
      if self.match_token(super::TokenType::Punctuation(super::PunctuationType::Comma)) {
        continue;
      }
      if self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::RegularBracketClose,
      )) {
        break;
      }
      let comma = self.at();
      return Err(ast::NodeError {
        message: "Se esperaba una coma (obj)".to_string(),
        location: comma.location,
      });
    }
    ast::Node::Object(ast::NodeObject {
      properties,
      location: open_brace.location,
    })
    .into()
  }
  fn parse_object_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
    let token = self.eat();
    match token.token_type {
      super::TokenType::StringLiteral => {
        let key = token.value;
        self.expect(
          super::TokenType::Punctuation(super::PunctuationType::DoubleDot),
          "Se esperaba dos puntos",
        )?;
        let value = self.parse_expr()?;
        return Ok(ast::NodeProperty::Property(key, value));
      }
      super::TokenType::Identifier | super::TokenType::Keyword(_) => {
        let key = &token.value;
        let colon = self.eat();
        if colon.token_type == super::TokenType::Error {
          return Err(ast::NodeError {
            message: "Se esperaba dos puntos".to_string(),
            location: colon.location,
          });
        }
        // the key is a variable name and value is an identifier
        if colon.token_type == super::TokenType::Punctuation(super::PunctuationType::Comma)
          || colon.token_type
            == super::TokenType::Punctuation(super::PunctuationType::RegularBracketClose)
        {
          self.index -= 1;
          return Ok(ast::NodeProperty::Property(
            key.clone(),
            ast::Node::Identifier(ast::NodeIdentifier {
              name: token.value,
              location: token.location,
            }),
          ));
        }
        if colon.token_type != super::TokenType::Punctuation(super::PunctuationType::DoubleDot) {
          return Err(ast::NodeError {
            message: "Se esperaba dos puntos".to_string(),
            location: colon.location,
          });
        }
        let value = self.parse_expr()?;
        return Ok(ast::NodeProperty::Property(key.clone(), value));
      }
      super::TokenType::Punctuation(p) => {
        if p == super::PunctuationType::QuadrateBracketOpen {
          let expr = self.parse_expr();
          self.expect(
            super::TokenType::Punctuation(super::PunctuationType::QuadrateBracketClose),
            "Se esperaba un corchete cuadrado de cierre (pop)",
          )?;
          let key = expr?;
          self.expect(
            super::TokenType::Punctuation(super::PunctuationType::DoubleDot),
            "Se esperaba dos puntos",
          )?;
          let value = self.parse_expr()?;
          return Ok(ast::NodeProperty::Dynamic(key, value));
        }
        if p == super::PunctuationType::Dot {
          self.expect(
            super::TokenType::Punctuation(super::PunctuationType::Dot),
            "Se esperaba un punto",
          )?;
          let data = self.parse_expr()?;
          return Ok(ast::NodeProperty::Iterable(data));
        }

        return Err(ast::NodeError {
          message: "Se esperaba un clave para la propiedad del objeto".to_string(),
          location: token.location,
        });
      }
      _ => {
        return Err(ast::NodeError {
          message: "Se esperaba un clave para la propiedad del objeto".to_string(),
          location: token.location,
        });
      }
    }
  }
  fn parse_array_expr(&mut self) -> Result<ast::Node, NodeError> {
    let open_bracket = self.eat();
    let mut elements = util::List::new();

    while !(self.is_eof()
      || self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::QuadrateBracketClose,
      )))
    {
      let element = self.parse_array_property()?;
      elements.push(element);
      if self.match_token(super::TokenType::Punctuation(super::PunctuationType::Comma)) {
        continue;
      }
      if self.match_token(super::TokenType::Punctuation(
        super::PunctuationType::QuadrateBracketClose,
      )) {
        break;
      }
      let comma = self.at();
      return Err(ast::NodeError {
        message: "Se esperaba una coma (util::Lista)".to_string(),
        location: comma.location,
      });
    }
    ast::Node::Array(ast::NodeArray {
      elements,
      location: open_bracket.location,
    })
    .into()
  }
  fn parse_array_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
    let token = self.at();
    match token.token_type {
      super::TokenType::Punctuation(p) => {
        if p == super::PunctuationType::Dot {
          self.eat();
          self.expect(
            super::TokenType::Punctuation(super::PunctuationType::Dot),
            "Se esperaba un punto",
          )?;
          let data = self.parse_expr()?;
          Ok(ast::NodeProperty::Iterable(data))
        } else {
          return Err(ast::NodeError {
            message: "Se esperaba un valor para la Lista".to_string(),
            location: token.location,
          });
        }
      }
      _ => {
        let element = self.parse_expr()?;
        Ok(ast::NodeProperty::Indexable(element))
      }
    }
  }
}
