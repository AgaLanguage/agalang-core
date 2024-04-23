pub mod ast;
use util::Token;

use crate::{internal::errors::ErrorTypes, util::{split_meta, to_cyan}};

use super::lexer::TokenType;

fn node_error(node: &ast::Node) -> ErrorTypes {
    let line;
    let column_node;
    let meta;
    let message;
    match node {
        ast::Node::Error(error) => {
            line = error.line + 1;
            column_node = error.column + 1;
            meta = error.file.clone();
            message = error.message.clone();
        }
        _ => {
            line = 1;
            column_node = 1;
            meta = "<indeterminado>".to_owned();
            message = "a ocurrido un error".to_owned();
        }
    }

    let (file, data_line, node_value) = split_meta(&meta);

    let column = column_node + node_value.len();

    let str_line = line.to_string();
    let str_init = " ".repeat(str_line.len());

    let cyan_line = to_cyan("|");
    let cyan_arrow = to_cyan("-->");

    let indicator: String = if node_value.len() > 0 {
        format!("{}^", "-".repeat(node_value.len()))
    } else {
        "^".to_string()
    };
    let lines = [
        format!("{}", message),
        format!("{}{cyan_arrow} {}:{}:{}", str_init, file, line, column),
        format!("{} {cyan_line}", str_init),
        format!("{} {cyan_line} {}", to_cyan(&str_line), data_line),
        format!(
            "{} {cyan_line} {}{}",
            str_init,
            " ".repeat(column_node - 1),
            to_cyan(&indicator)
        ),
        format!("{} {cyan_line}", str_init),
    ];
    let joined = lines.join("\n");
    ErrorTypes::StringError(joined)
}
pub struct Parser {
    source: String,
    tokens: Vec<Token<TokenType>>,
    index: usize,
    file_name: String,
}
impl Parser {
    pub fn new(source: String, ref file_name: String) -> Parser {
        let tokens = super::tokenizer(source.clone(), file_name.clone());
        Parser {
            source: source.clone(),
            tokens,
            index: 0,
            file_name: file_name.clone(),
        }
    }
    fn at(&mut self) -> util::Token<super::lexer::TokenType> {
        let token = self.tokens.get(self.index);
        if token.is_none() {
            return util::Token::<super::lexer::TokenType> {
                token_type: super::lexer::TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position: util::Position { column: 0, line: 0 },
                meta: self.file_name.clone(),
            };
        }
        let token = token.unwrap();
        util::Token::<super::lexer::TokenType> {
            token_type: token.token_type,
            value: token.value.clone(),
            position: util::Position {
                column: token.position.column,
                line: token.position.line,
            },
            meta: token.meta.clone(),
        }
    }
    fn eat(&mut self) -> util::Token<super::lexer::TokenType> {
        let token = self.at();
        self.index += 1;
        token
    }
    fn expect(
        &mut self,
        token_type: super::lexer::TokenType,
        err: &str,
    ) -> util::Token<super::lexer::TokenType> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        if token.is_none() {
            return util::Token::<super::lexer::TokenType> {
                token_type: super::lexer::TokenType::Error,
                value: err.to_string(),
                position: util::Position { column: 0, line: 0 },
                meta: self.file_name.clone(),
            };
        }
        let token = token.unwrap();
        if token.token_type != token_type {
            return util::Token::<super::lexer::TokenType> {
                token_type: super::lexer::TokenType::Error,
                value: err.to_string(),
                position: util::Position {
                    column: token.position.column,
                    line: token.position.line,
                },
                meta: self.file_name.clone(),
            };
        }
        util::Token::<super::lexer::TokenType> {
            token_type: token.token_type,
            value: token.value.clone(),
            position: util::Position {
                column: token.position.column,
                line: token.position.line,
            },
            meta: token.meta.clone(),
        }
    }
    pub fn produce_ast(&mut self, is_function: bool) -> ast::NodeProgram {
        let mut program = ast::NodeProgram {
            body: Vec::new(),
            node_type: ast::NodeType::Program,
            column: 0,
            line: 0,
            file: self.file_name.clone(),
        };
        //let functions: Vec<Box<dyn ast::Node>> = Vec::new();
        //let code: Vec<Box<dyn ast::Node>> = Vec::new();

        while self.index < self.tokens.len() {
            let (stmt, consumed) = self.parse_stmt(false, false, true);
            if let Some(stmt) = stmt {
                match stmt {
                    ast::Node::Error(error) => {
                        let node = &ast::Node::Error(error);
                        crate::internal::errors::throw_error(
                            crate::internal::errors::ErrorNames::SyntaxError,
                            node_error(node),
                        );
                        return program;
                    }
                    _ => {
                        program.body.push(stmt);
                    }
                }
            }
            self.index += consumed;
        }
        program
    }

    fn parse_stmt(
        &mut self,
        is_function: bool,
        is_class_decl: bool,
        is_global_scope: bool,
    ) -> (Option<ast::Node>, usize) {
        let token = self.at();
        match token.token_type {
            super::lexer::TokenType::EOF => {
                return (None, 1);
            }
            super::lexer::TokenType::Error => {
                return (
                    Some(ast::Node::Error(ast::NodeError {
                        message: token.value,
                        node_type: ast::NodeType::Error,
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    })),
                    1,
                );
            }
            super::lexer::TokenType::StringLiteral => {
                let node = ast::NodeStringLiteral {
                    value: token.value.clone(),
                    node_type: ast::NodeType::StringLiteral,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta.clone(),
                };
                return (Some(ast::Node::StringLiteral(node)), 1);
            }
            super::lexer::TokenType::NumberLiteral => {
                let node = ast::NodeNumberLiteral {
                    value: token.value.parse().unwrap(),
                    node_type: ast::NodeType::NumberLiteral,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta.clone(),
                };
                return (Some(ast::Node::NumberLiteral(node)), 1);
            }
            super::lexer::TokenType::Keyword => {
                if token.value == "def" || token.value == "const" {
                    return self.parse_var_decl();
                } else {
                    return (None, 1);
                }
            }
            _ => {
                return self.parse_expr();
            }
        }
    }

    fn parse_var_decl(&mut self) -> (Option<ast::Node>, usize) {
        let token = self.eat();
        let is_const = token.value == "const";
        let mut consumed = 1;

        let identifier = self.expect(
            super::lexer::TokenType::Identifier,
            "Se esperaba un identificador",
        );
        if identifier.token_type == super::lexer::TokenType::Error {
            let line = self.source.lines().nth(identifier.position.line).unwrap();
            let meta = format!("{}\0{}\0{} ", self.file_name, line, token.value);
            return (
                Some(ast::Node::Error(ast::NodeError {
                    message: identifier.value,
                    node_type: ast::NodeType::Error,
                    column: identifier.position.column,
                    line: identifier.position.line,
                    file: meta,
                })),
                consumed,
            );
        }
        consumed += 1;

        let equals_semicolon = self.eat();
        if equals_semicolon.token_type == super::lexer::TokenType::Punctuation
            && equals_semicolon.value == ";"
        {
            return (
                Some(ast::Node::VarDecl(ast::NodeVarDecl {
                    name: identifier.value.clone(),
                    value: None,
                    is_const,
                    node_type: ast::NodeType::VarDecl,
                    column: identifier.position.column,
                    line: identifier.position.line,
                    file: identifier.meta,
                })),
                consumed + 1,
            );
        }
        if equals_semicolon.token_type != super::lexer::TokenType::Operator
            || equals_semicolon.value != "="
        {
            let line = self.source.lines().nth(identifier.position.line).unwrap();
            let equals_line = token.position.line == identifier.position.line;
            // return a string with a var declaratio, example: "def value"
            let value = 
            if equals_line {
                format!("{}{}{}", token.value, " ".repeat(
                    (identifier.position.column as isize
                        - (token.position.column as isize
                            - (token.value.len() as isize - 1/* convert length to index value */))) as usize,
                ), identifier.value)
            } else {
                format!("{}{}", " ".repeat(identifier.position.column), identifier.value)
            };

            let meta = format!("{}\0{}\0{}", self.file_name, line, value);
            return (
                Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba un punto y coma".to_string(),
                    node_type: ast::NodeType::Error,
                    column: equals_semicolon.position.column,
                    line: equals_semicolon.position.line,
                    file: meta,
                })),
                consumed,
            );
        }
        consumed += 1;
        let (value, consumed_value) = self.parse_expr();
        if value.is_none() {
            return (
                Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba una expresión".to_string(),
                    node_type: ast::NodeType::Error,
                    column: equals_semicolon.position.column,
                    line: equals_semicolon.position.line,
                    file: equals_semicolon.meta.clone(),
                })),
                consumed,
            );
        }
        let value = value.unwrap();
        consumed += consumed_value;
        let semicolon = self.expect(
            super::lexer::TokenType::Punctuation,
            "Se esperaba un punto y coma",
        );
        if semicolon.token_type == super::lexer::TokenType::Error || semicolon.value != ";" {
            return (
                Some(ast::Node::Error(ast::NodeError {
                    message: semicolon.value,
                    node_type: ast::NodeType::Error,
                    column: semicolon.position.column,
                    line: semicolon.position.line,
                    file: semicolon.meta,
                })),
                consumed,
            );
        }
        consumed += 1;
        return (
            Some(ast::Node::VarDecl(ast::NodeVarDecl {
                name: identifier.value.clone(),
                value: Some(Box::new(value)),
                is_const,
                node_type: ast::NodeType::VarDecl,
                column: identifier.position.column,
                line: identifier.position.line,
                file: identifier.meta,
            })),
            consumed,
        );
    }

    fn parse_expr(&self) -> (Option<ast::Node>, usize) {
        (Some(ast::Node::NumberLiteral(ast::NodeNumberLiteral {
            value: "1".to_string(),
            node_type: ast::NodeType::NumberLiteral,
            column: 1,
            line: 1,
            file: self.file_name.clone(),
        })), 1)
    }
}
