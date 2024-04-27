pub mod ast;
pub mod string;
use util::Token;

use crate::{
    internal::errors::ErrorTypes,
    util::{split_meta, to_cyan},
};

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
            meta = error.meta.clone();
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
    fn not_eof(&self) -> bool {
        self.index < self.tokens.len()
    }
    fn at(&mut self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        if token.is_none() {
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position: util::Position { column: 0, line: 0 },
                meta: self.file_name.clone(),
            };
        }
        let token = token.unwrap();
        util::Token::<TokenType> {
            token_type: token.token_type,
            value: token.value.clone(),
            position: util::Position {
                column: token.position.column,
                line: token.position.line,
            },
            meta: token.meta.clone(),
        }
    }
    fn eat(&mut self) -> util::Token<TokenType> {
        let token = self.at();
        self.index += 1;
        token
    }
    fn expect(&mut self, token_type: TokenType, err: &str) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        if token.is_none() {
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: err.to_string(),
                position: util::Position { column: 0, line: 0 },
                meta: self.file_name.clone(),
            };
        }
        let token = token.unwrap();
        if token.token_type != token_type {
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: err.to_string(),
                position: util::Position {
                    column: token.position.column,
                    line: token.position.line,
                },
                meta: self.file_name.clone(),
            };
        }
        util::Token::<TokenType> {
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
            column: 0,
            line: 0,
            file: self.file_name.clone(),
        };
        //let functions: Vec<Box<dyn ast::Node>> = Vec::new();
        //let code: Vec<Box<dyn ast::Node>> = Vec::new();

        while self.not_eof() {
            let stmt = self.parse_stmt(false, false, true);
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
        }
        program
    }
    fn parse_stmt(
        &mut self,
        is_function: bool,
        is_class_decl: bool,
        is_global_scope: bool,
    ) -> Option<ast::Node> {
        let token = self.at();
        println!("{}: {}", token.token_type, token.value);
        match token.token_type {
            TokenType::EOF => {
                return None;
            }
            TokenType::Error => {
                return Some(ast::Node::Error(ast::NodeError {
                    message: token.value,
                    column: token.position.column,
                    line: token.position.line,
                    meta: token.meta,
                }));
            }
            TokenType::Keyword => {
                if token.value == "def" || token.value == "const" {
                    return self.parse_var_decl();
                } else {
                    return None;
                }
            }
            _ => {
                return self.parse_expr();
            }
        }
    }
    fn parse_var_decl(&mut self) -> Option<ast::Node> {
        let token = self.eat();
        let is_const = token.value == "const";

        let identifier = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if identifier.token_type == TokenType::Error {
            let line = self.source.lines().nth(identifier.position.line).unwrap();
            let meta = format!("{}\0{}\0{} ", self.file_name, line, token.value);
            return Some(ast::Node::Error(ast::NodeError {
                    message: identifier.value,
                    column: identifier.position.column,
                    line: identifier.position.line,
                    meta,
                }));
        }

        let equals_semicolon = self.eat();
        if equals_semicolon.token_type == TokenType::Punctuation && equals_semicolon.value == ";" {
            return Some(ast::Node::VarDecl(ast::NodeVarDecl {
                    name: identifier.value.clone(),
                    value: None,
                    is_const,
                    column: identifier.position.column,
                    line: identifier.position.line,
                    file: identifier.meta,
                }));
        }
        if equals_semicolon.token_type != TokenType::Operator || equals_semicolon.value != "=" {
            let line = self.source.lines().nth(identifier.position.line).unwrap();
            let equals_line = token.position.line == identifier.position.line;
            // return a string with a var declaratio, example: "def value"
            let value = if equals_line {
                format!("{}{}{}", token.value, " ".repeat(
                    (identifier.position.column as isize
                        - (token.position.column as isize
                            - (token.value.len() as isize - 1/* convert length to index value */))) as usize,
                ), identifier.value)
            } else {
                format!(
                    "{}{}",
                    " ".repeat(identifier.position.column),
                    identifier.value
                )
            };

            let meta = format!("{}\0{}\0{}", self.file_name, line, value);
            return Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba un punto y coma".to_string(),
                    column: equals_semicolon.position.column,
                    line: equals_semicolon.position.line,
                    meta,
                }));
        }
        let value = self.parse_expr();
        if value.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba una expresión".to_string(),
                    column: equals_semicolon.position.column,
                    line: equals_semicolon.position.line,
                    meta: equals_semicolon.meta.clone(),
                }));
        }
        let value = value.unwrap();
        if value.is_error() {
            return Some(value);
        }
        let semicolon = self.expect(TokenType::Punctuation, "");
        if semicolon.token_type == TokenType::Error || semicolon.value != ";" {
            let line = self.source.lines().nth(semicolon.position.line).unwrap();
            return Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba un punto y coma".to_string(),
                    column: semicolon.position.column,
                    line: semicolon.position.line,
                    meta: format!("{}\0{}\0{}", semicolon.meta, line, semicolon.value),
                }));
        }
        return Some(ast::Node::VarDecl(ast::NodeVarDecl {
                name: identifier.value.clone(),
                value: Some(Box::new(value)),
                is_const,
                column: identifier.position.column,
                line: identifier.position.line,
                file: identifier.meta,
            }));
    }
    fn parse_expr(&mut self) -> Option<ast::Node> {
        self.parse_literal_expr()
    }
    fn parse_literal_expr(&mut self) -> Option<ast::Node> {
        let token = self.eat();
        match token.token_type {
            TokenType::Identifier => {
                return Some(ast::Node::Identifier(ast::NodeIdentifier {
                        name: token.value.clone(),
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    }));
            }
            TokenType::NumberLiteral => {
                return Some(ast::Node::Number(ast::NodeNumber {
                        base: 10,
                        value: token.value.clone(),
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    }));
            }
            TokenType::Number => {
                let data = token.value.split("$").collect::<Vec<&str>>()[1];
                let base_value = data.split("~").collect::<Vec<&str>>();
                let base = base_value[0].parse::<i8>().unwrap();
                let value = base_value[1].to_string();
                return Some(ast::Node::Number(ast::NodeNumber {
                        base,
                        value,
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    }));
            }
            TokenType::StringLiteral => {
                return Some(ast::Node::String(ast::NodeString {
                        value: vec![ast::StringData::Str(token.value)],
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    }));
            }
            TokenType::String => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                let node = string::complex_string(token, line);
                match node {
                    Ok(node) => {
                        return Some(ast::Node::String(node));
                    }
                    Err(error) => {
                        return Some(ast::Node::Error(error));
                    }
                }
            }
            TokenType::Punctuation => match token.value.as_str() {
                "{" => {
                    let expr = self.parse_object_expr();
                    return Some(expr.unwrap());
                }
                "(" => {
                    let expr = self.parse_expr();
                    if expr.is_none() {
                        return Some(ast::Node::Error(ast::NodeError {
                                message: "Se esperaba una expresión".to_string(),
                                column: token.position.column,
                                line: token.position.line,
                                meta: token.meta,
                            }));
                    }
                    let expr = expr.unwrap();
                    let close_paren = self.expect(TokenType::Punctuation, "");
                    if close_paren.token_type == TokenType::Error || close_paren.value != ")" {
                        return Some(ast::Node::Error(ast::NodeError {
                                message: "Se esperaba un paréntesis de cierre".to_string(),
                                column: close_paren.position.column,
                                line: close_paren.position.line,
                                meta: close_paren.meta,
                            }));
                    }
                    return Some(expr);
                }
                _ => {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return Some(ast::Node::Error(ast::NodeError {
                            message: "Se esperaba una expresión".to_string(),
                            column: token.position.column,
                            line: token.position.line,
                            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                        }));
                }
            },
            _ => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Some(ast::Node::Error(ast::NodeError {
                        message: "Se esperaba una expresión".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                    }));
            }
        }
    }
    fn parse_object_expr(&mut self) -> Option<ast::Node> {
        let open_brace = self.eat();
        let mut properties: Vec<ast::NodeProperty> = Vec::new();

        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "}")
        {
            let property = self.parse_object_property("}");
            if property.is_err() {
                return Some(ast::Node::Error(property.err().unwrap()));
            }
            let property = property.ok().unwrap();
            properties.push(property);
        }
        let close_brace = self.expect(
            TokenType::Punctuation,
            "Se esperaba un paréntesis de cierre",
        );
        if close_brace.token_type == TokenType::Error || close_brace.value != "}" {
            return Some(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba un paréntesis de cierre".to_string(),
                    column: close_brace.position.column,
                    line: close_brace.position.line,
                    meta: close_brace.meta,
                }));
        }
        Some(ast::Node::Object(ast::NodeObject {
                properties,
                column: open_brace.position.column,
                line: open_brace.position.line,
                file: open_brace.meta,
            }))
    }
    fn parse_object_property(
        &mut self,
        close_char: &str,
    ) -> Result<ast::NodeProperty, ast::NodeError> {
        let token = self.eat();
        match token.token_type {
            TokenType::StringLiteral => {
                let key = token.value;
                let colon = self.expect(TokenType::Punctuation, "");
                if colon.token_type == TokenType::Error || colon.value != ":" {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba dos puntos".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                let value = self.parse_expr();
                if value.is_none() {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba una expresión".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                let value = value.unwrap();
                return Ok(ast::NodeProperty::Property(key, value));
            }
            TokenType::Identifier => {
                let key = &token.value;
                let colon = self.expect(TokenType::Punctuation, "Se esperaba dos puntos");
                if colon.token_type == TokenType::Error {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: colon.value.clone(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                // the key is a variable name and value is an identifier
                if colon.value == "," || colon.value == close_char.to_string() {
                    return Ok(ast::NodeProperty::Property(
                        key.clone(),
                        ast::Node::Identifier(ast::NodeIdentifier {
                            name: token.value,
                            column: token.position.column,
                            line: token.position.line,
                            file: token.meta,
                        }),
                    ));
                }
                let value = self.parse_expr();
                if value.is_none() {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba una expresión".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                let value = value.unwrap();
                return Ok(ast::NodeProperty::Property(key.clone(), value));
            }
            TokenType::Punctuation => {
                if token.value == "[" {
                    let expr = self.parse_expr();
                    if expr.is_none() {
                        let line = self.source.lines().nth(token.position.line).unwrap();
                        return Err(ast::NodeError {
                            message: "Se esperaba una expresión".to_string(),
                            column: token.position.column,
                            line: token.position.line,
                            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                        });
                    }
                    let expr = expr.unwrap();
                    let close_bracket = self.expect(TokenType::Punctuation, "");
                    if close_bracket.token_type == TokenType::Error || close_bracket.value != "]" {
                        let line = self
                            .source
                            .lines()
                            .nth(close_bracket.position.line)
                            .unwrap();
                        return Err(ast::NodeError {
                            message: "Se esperaba un corchete de cierre".to_string(),
                            column: close_bracket.position.column,
                            line: close_bracket.position.line,
                            meta: format!(
                                "{}\0{}\0{}",
                                close_bracket.meta, line, close_bracket.value
                            ),
                        });
                    }
                    let key = expr;
                    let colon = self.expect(TokenType::Punctuation, "");
                    if colon.token_type == TokenType::Error || colon.value != ":" {
                        let line = self.source.lines().nth(colon.position.line).unwrap();
                        return Err(ast::NodeError {
                            message: "Se esperaba dos puntos".to_string(),
                            column: colon.position.column,
                            line: colon.position.line,
                            meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                        });
                    }
                    let value = self.parse_expr();
                    if value.is_none() {
                        let line = self.source.lines().nth(colon.position.line).unwrap();
                        return Err(ast::NodeError {
                            message: "Se esperaba una expresión".to_string(),
                            column: colon.position.column,
                            line: colon.position.line,
                            meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                        });
                    }
                    let value = value.unwrap();
                    return Ok(ast::NodeProperty::Dynamic(key, value));
                }
                if token.value == "." {
                    let dot = self.expect(TokenType::Punctuation, "");
                    if dot.token_type == TokenType::Error || dot.value != "." {
                        let line = self.source.lines().nth(dot.position.line).unwrap();
                        return Err(ast::NodeError {
                            message: "Se esperaba un punto".to_string(),
                            column: dot.position.column,
                            line: dot.position.line,
                            meta: format!("{}\0{}\0{}", dot.meta, line, dot.value),
                        });
                    }
                    let iterable =
                        self.expect(TokenType::Identifier, "Se esperaba un identificador");
                    if iterable.token_type == TokenType::Error {
                        let line = self.source.lines().nth(iterable.position.line).unwrap();
                        return Err(ast::NodeError {
                            message: iterable.value,
                            column: iterable.position.column,
                            line: iterable.position.line,
                            meta: format!("{}\0{}\0{}", iterable.meta, line, token.value),
                        });
                    }
                    return Ok(ast::NodeProperty::Iterable(ast::Node::Identifier(
                        ast::NodeIdentifier {
                            name: iterable.value,
                            column: iterable.position.column,
                            line: iterable.position.line,
                            file: iterable.meta,
                        },
                    )));
                }
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Err(ast::NodeError {
                    message: "Se esperaba un clave para la propiedad del objeto".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                });
            }
            _ => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Err(ast::NodeError {
                    message: "Se esperaba un clave para la propiedad del objeto".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                });
            }
        }
    }
    //fn parse_array_expr(&mut self) -> (Option<ast::Node>, usize) {}
}
