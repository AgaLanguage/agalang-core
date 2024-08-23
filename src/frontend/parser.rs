pub mod ast;
pub mod string;
use ast::NodeBlock;
use util::{List, Token};

use crate::{
    internal,
    util::{split_meta, to_cyan},
};

use super::lexer::{KeywordsType, OperatorType, PunctuationType, TokenType};

const ASSIGNMENT_MODIFICATOR: [&str; 9] = ["+", "-", "*", "/", "%", "~", "&&", "||", "??"];
const COMPARISON: [&str; 4] = ["=", "!", "<", ">"];
const MISSING_TOKEN: &str = "\x1b[81mToken desaparecido\x1b[0m";

struct SemiToken {
    value: String,
    column: usize,
    line: usize,
}

pub fn node_error(error: &ast::NodeError) -> internal::ErrorTypes {
    let line: usize;
    let column_node: usize;
    let meta: &str;
    let message: &str;

    if error.message == MISSING_TOKEN {
        line = 0;
        column_node = 0;
        meta = &error.meta;
        message = MISSING_TOKEN;
    } else {
        line = error.line + 1;
        column_node = error.column + 1;
        meta = &error.meta;
        message = &error.message;
    }

    let (file, data_line, node_value) = split_meta(&meta);

    let column = column_node + node_value.len();

    let str_line = line.to_string();
    let str_init = " ".repeat(str_line.len());

    let cyan_line = to_cyan("|");
    let cyan_arrow = to_cyan("-->");

    let indicator = if node_value.len() > 0 {
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
    internal::ErrorTypes::StringError(joined)
}

pub struct Parser {
    source: String,
    tokens: Vec<Token<TokenType>>,
    index: usize,
    file_name: String,
}
impl Parser {
    pub fn new(source: String, file_name: &String) -> Parser {
        let tokens = super::tokenizer(source.clone(), file_name.clone());
        Parser {
            source: source.clone(),
            tokens,
            index: 0,
            file_name: file_name.clone(),
        }
    }
    fn not_eof(&mut self) -> bool {
        self.index < self.tokens.len()
    }
    fn prev(&self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index - 1);
        if token.is_none() {
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: MISSING_TOKEN.to_string(),
                position: util::Position { column: 0, line: 0 },
                meta: format!("{}\0{}\0{}", &self.file_name, MISSING_TOKEN, MISSING_TOKEN),
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
    fn at(&self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        if token.is_none() {
            let position = self.prev().position;
            let line = self.source.lines().nth(position.line).unwrap();
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position,
                meta: format!(
                    "{}\0{}\0{}",
                    &self.file_name,
                    line,
                    " ".repeat(position.column)
                ),
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
    fn next(&self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index + 1);
        if token.is_none() {
            let position = self.prev().position;
            let line = self.source.lines().nth(position.line).unwrap();
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position,
                meta: format!(
                    "{}\0{}\0{}",
                    &self.file_name,
                    line,
                    " ".repeat(position.column)
                ),
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
    fn expect(&mut self, token_type: TokenType, err: &str) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        if token.is_none() {
            let position = self.prev().position;
            let line = self.source.lines().nth(position.line).unwrap();
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: err.to_string(),
                position,
                meta: format!(
                    "{}\0{}\0{}",
                    &self.file_name,
                    line,
                    " ".repeat(position.column)
                ),
            };
        }
        let token = token.unwrap();
        if token.token_type != token_type {
            let line = self.source.lines().nth(token.position.line).unwrap();
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: err.to_string(),
                position: util::Position {
                    column: token.position.column,
                    line: token.position.line,
                },
                meta: format!(
                    "{}\0{}\0{}",
                    &self.file_name,
                    line,
                    " ".repeat(token.position.column)
                ),
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
    pub fn produce_ast(&mut self) -> ast::Node {
        let body = self.parse_block(true, false, false, TokenType::EOF);
        if body.is_err() {
            return ast::Node::Error(body.err().unwrap());
        }
        let body = body.ok().unwrap();
        ast::Node::Program(ast::NodeProgram {
            body,
            column: 0,
            line: 0,
            file: self.file_name.clone(),
        })
    }
    fn parse_stmt(
        &mut self,
        is_global_scope: bool,
        is_function: bool,
        is_loop: bool,
    ) -> Option<ast::Node> {
        let token = self.at();
        match token.token_type {
            TokenType::EOF => {
                self.eat();
                None
            }
            TokenType::Error => {
                return Some(ast::Node::Error(ast::NodeError {
                    message: token.value,
                    column: token.position.column,
                    line: token.position.line,
                    meta: token.meta,
                }));
            }
            TokenType::Keyword(key) => match key {
                KeywordsType::Definir | KeywordsType::Constante => Some(self.parse_var_decl()),
                KeywordsType::Mientras
                | KeywordsType::Hacer
                | KeywordsType::Si
                | KeywordsType::Funcion
                | KeywordsType::Intentar
                | KeywordsType::Clase
                | KeywordsType::Para => Some(self.parse_keyword_value(is_function, is_loop)),
                KeywordsType::Retornar | KeywordsType::Continuar | KeywordsType::Romper => {
                    Some(self.parse_simple_decl(is_function, is_loop))
                }
                KeywordsType::Exportar => Some(self.parse_export_decl(is_global_scope)),
                KeywordsType::Importar => Some(self.parse_import_decl(is_global_scope)),
                KeywordsType::Lanzar => Some(self.parse_throw_decl()),
                _ => {
                    self.eat();
                    None
                }
            },
            _ => Some(self.parse_stmt_expr()),
        }
    }
    fn parse_throw_decl(&mut self) -> ast::Node {
        let token = self.eat(); // lanzar
        let expr = self.parse_expr();
        if expr.is_error() {
            return expr;
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: format!(
                    "Se esperaba un punto y coma ({})",
                    KeywordsType::Lanzar.to_string()
                ),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        ast::Node::Throw(ast::NodeValue {
            value: Box::new(expr),
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn parse_import_decl(&mut self, is_global_scope: bool) -> ast::Node {
        let token = self.eat(); // importar
        let path = self.expect(
            TokenType::StringLiteral,
            "Se esperaba una ruta de archivo, debe usar una cadena literal con '",
        );
        if path.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: path.value.clone(),
                column: path.position.column,
                line: path.position.line,
                meta: path.meta,
            });
        }
        let mut name = None;
        if self.at().token_type == TokenType::Keyword(KeywordsType::Como) {
            self.eat();
            let alias = self.expect(TokenType::Identifier, "Se esperaba un identificador");
            if alias.token_type == TokenType::Error {
                return ast::Node::Error(ast::NodeError {
                    message: alias.value.clone(),
                    column: alias.position.column,
                    line: alias.position.line,
                    meta: alias.meta,
                });
            }
            name = Some(alias.value.clone());
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: format!("Se esperaba un punto y coma ({})", path.value),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        if !is_global_scope {
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "No se puede importar fuera del ámbito global".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
            });
        }
        ast::Node::Import(ast::NodeImport {
            path: path.value.clone(),
            name,
            column: token.position.column,
            line: token.position.line,
            file: token.meta.clone(),
        })
    }
    fn parse_export_decl(&mut self, is_global_scope: bool) -> ast::Node {
        let token = self.eat(); // exportar
        let value = self.parse_export_value();
        if value.is_error() {
            return value;
        }
        if !is_global_scope {
            let line = self.source.lines().nth(token.position.line).unwrap();
            let error = ast::NodeError {
                message: "No se puede exportar fuera del ámbito global".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
            };
            let type_err = internal::ErrorNames::SyntaxError;
            let err = node_error(&error);
            let data = internal::error_to_string(&type_err, err);
            internal::print_warn(data);
            return value;
        }
        ast::Node::Export(ast::NodeValue {
            value: Box::new(value),
            column: token.position.column,
            line: token.position.line,
            file: token.meta.clone(),
        })
    }
    fn parse_export_value(&mut self) -> ast::Node {
        let token = self.at();
        match token.token_type {
            TokenType::Keyword(KeywordsType::Definir | KeywordsType::Constante) => {
                self.parse_var_decl()
            }
            TokenType::Keyword(KeywordsType::Funcion | KeywordsType::Clase) => {
                self.parse_keyword_value(false, false)
            }
            TokenType::Keyword(KeywordsType::Nombre) => self.parse_name_decl(),
            _ => {
                self.eat();
                let line = self.source.lines().nth(token.position.line).unwrap();
                let (message, value) = if token.token_type == TokenType::Error {
                    (token.value, "".to_string())
                } else {
                    ("Se esperaba un valor exportable".to_string(), token.value)
                };
                ast::Node::Error(ast::NodeError {
                    message,
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", &self.file_name, line, value),
                })
            }
        }
    }
    fn parse_name_decl(&mut self) -> ast::Node {
        let token = self.eat(); // nombre
        let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if name.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: name.value.clone(),
                column: name.position.column,
                line: name.position.line,
                meta: name.meta,
            });
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: format!(
                    "Se esperaba un punto y coma ({})",
                    KeywordsType::Nombre.to_string()
                ),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        ast::Node::Name(ast::NodeIdentifier {
            name: name.value.clone(),
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn parse_class_prop(
        &mut self,
        is_static: bool,
        is_const: bool,
        is_public: bool,
    ) -> Result<ast::NodeClassProperty, ast::NodeError> {
        let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if name.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: name.value.clone(),
                column: name.position.column,
                line: name.position.line,
                meta: name.meta,
            });
        }
        let token = self.at();
        if token.token_type == TokenType::Error {
            self.eat();
            return Err(ast::NodeError {
                message: token.value.clone(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta,
            });
        }
        let is_static_bit: u8 = if is_static { 1 } else { 0 };
        let is_const: u8 = if is_const { 1 << 1 } else { 0 };
        let is_public_bit: u8 = if is_public { 1 << 2 } else { 0 };
        let meta: u8 = is_static_bit | is_public_bit | is_const;
        if token.token_type == TokenType::Punctuation(PunctuationType::SemiColon) {
            return Ok(ast::NodeClassProperty {
                name: name.value.clone(),
                value: None,
                meta,
            });
        }
        let value: ast::Node =
            if token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen) {
                let params = self.parse_arguments_expr();
                if params.is_err() {
                    return Err(params.err().unwrap());
                }
                let params = params.ok().unwrap();
                let block = self.parse_block_expr(false, false);
                if block.is_err() {
                    return Err(block.err().unwrap());
                }
                let body = block.ok().unwrap();
                ast::Node::Function(ast::NodeFunction {
                    name: name.value.clone(),
                    params,
                    body,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta.clone(),
                })
            } else if token.token_type == TokenType::Operator(OperatorType::Equals) {
                self.eat();
                self.parse_expr()
            } else {
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Err(ast::NodeError {
                    message: "Se esperaba un valor".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                });
            };
        if value.is_error() {
            return Err(value.get_error().unwrap().clone());
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: format!("Se esperaba un punto y coma ({})", name.value),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        Ok(ast::NodeClassProperty {
            name: name.value.clone(),
            value: Some(value.to_box()),
            meta,
        })
    }
    fn parse_class_decl(&mut self) -> ast::Node {
        let token = self.eat(); // clase
        let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if name.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: name.value.clone(),
                column: name.position.column,
                line: name.position.line,
                meta: name.meta,
            });
        }
        let open_brace = self.expect(
            TokenType::Punctuation(PunctuationType::RegularBracketOpen),
            "",
        );
        if open_brace.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un corchete de apertura".to_string(),
                column: open_brace.position.column,
                line: open_brace.position.line,
                meta: open_brace.meta,
            });
        }
        let mut body: List<ast::NodeClassProperty> = List::new();
        while self.not_eof()
            && !(self.at().token_type
                == TokenType::Punctuation(PunctuationType::RegularBracketClose))
        {
            let modifier = self.get_modifier();

            if modifier.is_err() {
                return ast::Node::Error(modifier.err().unwrap());
            }

            let (is_static, is_const, is_public) = modifier.ok().unwrap();

            let prop = self.parse_class_prop(is_static, is_const, is_public);
            if prop.is_err() {
                return ast::Node::Error(prop.err().unwrap());
            }
            let prop = prop.ok().unwrap();
            body.push(prop);
        }
        let close_brace = self.expect(
            TokenType::Punctuation(PunctuationType::RegularBracketClose),
            "",
        );
        if close_brace.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un corchete de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: close_brace.meta,
            });
        }
        ast::Node::Class(ast::NodeClass {
            name: name.value.clone(),
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn get_modifier(&mut self) -> Result<(bool, bool, bool), ast::NodeError> {
        let mut is_static = false;
        let mut is_const = false;
        let mut is_public = false;
        while self.not_eof() {
            let token = self.at();
            if token.token_type == TokenType::Error {
                self.eat();
                return Err(ast::NodeError {
                    message: token.value.clone(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: token.meta,
                });
            }
            if token.token_type == TokenType::Keyword(KeywordsType::Estatico) {
                if is_static {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Modificador duplicado".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                    });
                }
                is_static = true;
                self.eat();
                continue;
            }
            if token.token_type == TokenType::Keyword(KeywordsType::Publico) {
                if is_public {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Modificador duplicado".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                    });
                }
                is_public = true;
                self.eat();
                continue;
            }
            if token.token_type == TokenType::Keyword(KeywordsType::Constante) {
                if is_const {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Modificador duplicado".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                    });
                }
                is_const = true;
                self.eat();
                continue;
            }
            break;
        }
        Ok((is_static, is_const, is_public))
    }
    fn parse_simple_decl(&mut self, is_function: bool, is_loop: bool) -> ast::Node {
        let token = self.eat(); // TokenType::Keyword
                                // cont rom ret
        match token.token_type {
            TokenType::Keyword(KeywordsType::Retornar) => {
                if !is_function {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        message: "No se puede retornar fuera de una función".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                    });
                }
                let expr = self.parse_expr();
                if expr.is_error() {
                    return expr;
                }
                let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
                if semicolon.token_type == TokenType::Error {
                    return ast::Node::Error(ast::NodeError {
                        message: format!(
                            "Se esperaba un punto y coma ({})",
                            KeywordsType::Retornar.to_string()
                        ),
                        column: semicolon.position.column,
                        line: semicolon.position.line,
                        meta: semicolon.meta,
                    });
                }
                return ast::Node::Return(ast::NodeReturn {
                    value: Some(expr.to_box()),
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                });
            }
            TokenType::Keyword(KeywordsType::Romper | KeywordsType::Continuar) => {
                if !is_loop {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        message: "No se puede usar esta palabra clave fuera de un ciclo"
                            .to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                    });
                }
                let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
                if semicolon.token_type == TokenType::Error {
                    return ast::Node::Error(ast::NodeError {
                        message: format!("Se esperaba un punto y coma (Modificador de Bucle)"),
                        column: semicolon.position.column,
                        line: semicolon.position.line,
                        meta: semicolon.meta,
                    });
                }
                let action = if token.value == "cont" {
                    ast::NodeLoopEditType::Continue
                } else {
                    ast::NodeLoopEditType::Break
                };
                return ast::Node::LoopEdit(ast::NodeLoopEdit {
                    action,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                });
            }
            TokenType::Error => ast::Node::Error(ast::NodeError {
                message: token.value.clone(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta,
            }),
            _ => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                return ast::Node::Error(ast::NodeError {
                    message: "Token inesperado (simple)".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                });
            }
        }
    }
    fn parse_keyword_value(&mut self, is_function: bool, is_loop: bool) -> ast::Node {
        let token = self.at();
        match token.token_type {
            TokenType::Keyword(KeywordsType::Para) => self.parse_for_decl(is_function),
            TokenType::Keyword(KeywordsType::Mientras) => self.parse_while_decl(is_function),
            TokenType::Keyword(KeywordsType::Hacer) => self.parse_do_while_decl(is_function),
            TokenType::Keyword(KeywordsType::Si) => self.parse_if_decl(is_function, is_loop),
            TokenType::Keyword(KeywordsType::Funcion) => self.parse_function_decl(),
            TokenType::Keyword(KeywordsType::Intentar) => self.parse_try_decl(is_function, is_loop),
            TokenType::Keyword(KeywordsType::Clase) => self.parse_class_decl(),
            TokenType::Error => {
                self.eat();
                ast::Node::Error(ast::NodeError {
                    message: token.value.clone(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: token.meta,
                })
            }
            _ => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                ast::Node::Error(ast::NodeError {
                    message: "Token inesperado (keyword)".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", &self.file_name, line, token.value),
                })
            }
        }
    }
    fn parse_for_decl(&mut self, is_function: bool) -> ast::Node {
        let token = self.eat(); // para
        let open_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketOpen),
            "",
        );
        if open_paren.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de apertura".to_string(),
                column: open_paren.position.column,
                line: open_paren.position.line,
                meta: open_paren.meta,
            });
        }
        let init = self.parse_var_decl();
        let condition = self.parse_expr();
        if condition.is_error() {
            return condition;
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma (Para)".to_string(),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        let update = self.parse_expr();
        if update.is_error() {
            return update;
        }
        let close_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketClose),
            "",
        );
        if close_paren.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de cierre".to_string(),
                column: close_paren.position.column,
                line: close_paren.position.line,
                meta: close_paren.meta,
            });
        }
        let block = self.parse_block_expr(is_function, true);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        ast::Node::For(ast::NodeFor {
            init: Box::new(init),
            condition: Box::new(condition),
            update: Box::new(update),
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn parse_try_decl(&mut self, is_function: bool, is_loop: bool) -> ast::Node {
        let token = self.eat(); // intentar
        let block = self.parse_block_expr(is_function, is_loop);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        let catch_token = self.expect(
            TokenType::Keyword(KeywordsType::Capturar),
            "Se esperaba 'capturar'",
        );
        if catch_token.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba 'capturar'".to_string(),
                column: catch_token.position.column,
                line: catch_token.position.line,
                meta: catch_token.meta,
            });
        }
        let open_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketOpen),
            "",
        );
        if open_paren.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de apertura".to_string(),
                column: open_paren.position.column,
                line: open_paren.position.line,
                meta: open_paren.meta,
            });
        }
        let identifier = self.expect(TokenType::Identifier, "");
        if identifier.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: identifier.value.clone(),
                column: identifier.position.column,
                line: identifier.position.line,
                meta: identifier.meta,
            });
        }
        let close_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketClose),
            "",
        );
        if close_paren.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de cierre".to_string(),
                column: close_paren.position.column,
                line: close_paren.position.line,
                meta: close_paren.meta,
            });
        }
        let block = self.parse_block_expr(is_function, is_loop);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body_catch = block.ok().unwrap();
        let finally = if self.at().token_type == TokenType::Keyword(KeywordsType::Finalmente) {
            self.eat();
            let block = self.parse_block_expr(is_function, is_loop);
            if block.is_err() {
                return ast::Node::Error(block.err().unwrap());
            }
            Some(block.ok().unwrap())
        } else {
            None
        };
        let catch = (identifier.value.clone(), body_catch);
        ast::Node::Try(ast::NodeTry {
            body,
            catch,
            finally,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn parse_function_decl(&mut self) -> ast::Node {
        let token = self.eat(); // fn
        let name = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if name.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: name.value.clone(),
                column: name.position.column,
                line: name.position.line,
                meta: name.meta,
            });
        }
        let params = self.parse_arguments_expr();
        if params.is_err() {
            return ast::Node::Error(params.err().unwrap());
        }
        let params = params.ok().unwrap();
        let block = self.parse_block_expr(true, false);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        ast::Node::Function(ast::NodeFunction {
            name: name.value.clone(),
            params,
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        })
    }
    fn parse_arguments_expr(&mut self) -> Result<List<ast::NodeIdentifier>, ast::NodeError> {
        let open_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketOpen),
            "",
        );
        if open_paren.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: "Se esperaba un paréntesis de apertura".to_string(),
                column: open_paren.position.column,
                line: open_paren.position.line,
                meta: open_paren.meta,
            });
        }
        let mut params = List::new();
        while self.not_eof()
            && !(self.at().token_type
                == TokenType::Punctuation(PunctuationType::CircularBracketClose))
        {
            let param = self.expect(TokenType::Identifier, "Se esperaba un identificador");
            if param.token_type == TokenType::Error {
                return Err(ast::NodeError {
                    message: param.value.clone(),
                    column: param.position.column,
                    line: param.position.line,
                    meta: param.meta,
                });
            }
            params.push(ast::NodeIdentifier {
                name: param.value.clone(),
                column: param.position.column,
                line: param.position.line,
                file: param.meta,
            });
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
                self.eat();
                continue;
            }
            if comma.token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose) {
                break;
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return Err(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", self.file_name, line, comma.value),
            });
        }
        let close_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketClose),
            "Se esperaba un paréntesis de cierre",
        );
        if close_paren.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: "Se esperaba un paréntesis de cierre".to_string(),
                column: close_paren.position.column,
                line: close_paren.position.line,
                meta: close_paren.meta,
            });
        }
        Ok(params)
    }
    fn parse_if_decl(&mut self, is_function: bool, is_loop: bool) -> ast::Node {
        let token = self.eat(); // si
        let condition = self.parse_expr();
        if condition.is_error() {
            return condition;
        }
        let block = self.parse_block_expr(is_function, is_loop);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        let else_token = self.at(); // ent
        if else_token.token_type == TokenType::Keyword(KeywordsType::Entonces) {
            self.eat();
            let else_block = self.parse_block_expr(is_function, is_loop);
            if else_block.is_err() {
                return ast::Node::Error(else_block.err().unwrap());
            }
            let else_body = else_block.ok().unwrap();
            let else_body = if else_body.len() == 0 {
                None
            } else {
                Some(else_body)
            };
            return ast::Node::If(ast::NodeIf {
                condition: condition.to_box(),
                body,
                else_body,
                column: token.position.column,
                line: token.position.line,
                file: token.meta,
            });
        }
        return ast::Node::If(ast::NodeIf {
            condition: condition.to_box(),
            body,
            else_body: None,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
    }
    fn parse_do_while_decl(&mut self, is_function: bool) -> ast::Node {
        let token = self.eat(); // hacer
        let block = self.parse_block_expr(is_function, true);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        let while_token = self.expect(
            TokenType::Keyword(KeywordsType::Mientras),
            "Se esperaba la palabra clave 'mien'",
        );
        if while_token.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba la palabra clave 'mien'".to_string(),
                column: while_token.position.column,
                line: while_token.position.line,
                meta: while_token.meta,
            });
        }
        let condition = self.parse_expr();
        if condition.is_error() {
            return condition;
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: format!(
                    "Se esperaba un punto y coma ({})",
                    KeywordsType::Hacer.to_string()
                ),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        return ast::Node::DoWhile(ast::NodeWhile {
            condition: condition.to_box(),
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
    }
    fn parse_while_decl(&mut self, is_function: bool) -> ast::Node {
        let token = self.eat(); // mien
        let condition = self.parse_expr();
        if condition.is_error() {
            return condition;
        }
        let block = self.parse_block_expr(is_function, true);
        if block.is_err() {
            return ast::Node::Error(block.err().unwrap());
        }
        let body = block.ok().unwrap();
        return ast::Node::While(ast::NodeWhile {
            condition: condition.to_box(),
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
    }
    fn parse_block_expr(
        &mut self,
        in_function: bool,
        in_loop: bool,
    ) -> Result<NodeBlock, ast::NodeError> {
        let open_brace = self.at();
        if open_brace.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: "Se esperaba un bloque".to_string(),
                column: open_brace.position.column,
                line: open_brace.position.line,
                meta: open_brace.meta,
            });
        }
        if self.at().token_type != TokenType::Punctuation(PunctuationType::RegularBracketOpen) {
            let expr = self.parse_stmt(false, in_function, in_loop);
            if expr.is_none() {
                let line = self.source.lines().nth(open_brace.position.line).unwrap();
                return Err(ast::NodeError {
                    message: "Se esperaba un bloque".to_string(),
                    column: open_brace.position.column,
                    line: open_brace.position.line,
                    meta: format!("{}\0{}", self.file_name, line),
                });
            }
            let expr = expr.unwrap();
            if expr.is_error() {
                return Err(expr.get_error().unwrap().clone());
            }
            let mut body = List::new();
            body.push(expr);
            return Ok(ast::NodeBlock {
                body,
                in_function,
                in_loop,
            });
        }
        self.eat();
        let body = self.parse_block(
            false,
            in_function,
            in_loop,
            TokenType::Punctuation(PunctuationType::RegularBracketClose),
        );
        if body.is_err() {
            return Err(body.err().unwrap());
        }
        let body = body.ok().unwrap();
        let close_brace = self.expect(
            TokenType::Punctuation(PunctuationType::RegularBracketClose),
            "",
        );
        if close_brace.token_type == TokenType::Error {
            return Err(ast::NodeError {
                message: "Se esperaba un corchete de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: close_brace.meta,
            });
        }
        Ok(body)
    }
    fn parse_block(
        &mut self,
        is_global_scope: bool,
        is_function: bool,
        is_loop: bool,
        stop_with: TokenType,
    ) -> Result<NodeBlock, ast::NodeError> {
        let mut body = List::new();
        let mut error = None;
        let mut functions = Vec::new();
        let mut code = Vec::new();
        while self.not_eof() && !(self.at().token_type == stop_with) && error.is_none() {
            let stmt = self.parse_stmt(is_global_scope, is_function, is_loop);
            if let Some(stmt) = stmt {
                match stmt {
                    ast::Node::Error(node) => error = Some(node),
                    ast::Node::Function(_) => functions.push(stmt),
                    ast::Node::Export(ref export) => match *export.value {
                        ast::Node::Function(_) => functions.push(stmt.clone()),
                        _ => code.push(stmt),
                    },
                    _ => code.push(stmt),
                }
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        body.append_vec(&mut functions);
        body.append_vec(&mut code);
        Ok(ast::NodeBlock {
            body,
            in_function: false,
            in_loop: false,
        })
    }
    fn parse_var_decl(&mut self) -> ast::Node {
        let token = self.eat();
        let is_const = token.value == "const";
        let mut semi_token = SemiToken {
            value: token.value,
            column: token.position.column,
            line: token.position.line,
        };

        let identifier = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if semi_token.line == identifier.position.line {
            semi_token.value += " "
                .repeat(identifier.position.column - semi_token.column)
                .as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.line = identifier.position.line;
        semi_token.column = identifier.position.column;
        if identifier.token_type == TokenType::Error {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return ast::Node::Error(ast::NodeError {
                message: identifier.value,
                column: semi_token.column,
                line: semi_token.line,
                meta,
            });
        }
        semi_token.value += identifier.value.as_str();

        let equals_semicolon = self.eat();
        if semi_token.line == equals_semicolon.position.line {
            semi_token.value += " "
                .repeat(equals_semicolon.position.column - semi_token.column)
                .as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.line = equals_semicolon.position.line;
        semi_token.column = equals_semicolon.position.column;
        if equals_semicolon.token_type == TokenType::Punctuation(PunctuationType::SemiColon) {
            return ast::Node::VarDecl(ast::NodeVarDecl {
                name: identifier.value.clone(),
                value: None,
                is_const,
                column: identifier.position.column,
                line: identifier.position.line,
                file: identifier.meta,
            });
        }
        if equals_semicolon.token_type != TokenType::Operator(OperatorType::Equals) {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return ast::Node::Error(ast::NodeError {
                message: format!("Se esperaba un punto y coma (variable e)"),
                column: semi_token.column,
                line: semi_token.line,
                meta,
            });
        }
        semi_token.value += equals_semicolon.value.as_str();

        let value = self.parse_expr();
        if semi_token.line == value.get_line() {
            semi_token.value += " ".repeat(value.get_column() - semi_token.column).as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.column = value.get_column();
        if value.is_error() {
            return value;
        }
        let semicolon = self.expect(TokenType::Punctuation(PunctuationType::SemiColon), "");
        if semi_token.line == semicolon.position.line {
            semi_token.value += " "
                .repeat(semicolon.position.column - semi_token.column)
                .as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.column += 1;
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: format!("Se esperaba un punto y coma (variable v)"),
                column: semi_token.column,
                line: semi_token.line,
                meta: semi_token.value,
            });
        }
        return ast::Node::VarDecl(ast::NodeVarDecl {
            name: identifier.value.clone(),
            value: Some(value.to_box()),
            is_const,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
    }
    fn parse_stmt_expr(&mut self) -> ast::Node {
        let node = self.parse_expr();
        if node.is_error() {
            return node;
        }
        let semicolon = self.expect(
            TokenType::Punctuation(PunctuationType::SemiColon),
            "Se esperaba un punto y coma",
        );
        if semicolon.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: semicolon.value,
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: semicolon.meta,
            });
        }
        node
    }
    fn parse_expr(&mut self) -> ast::Node {
        let left = self.parse_math_lineal_expr();
        if left.is_error() {
            return left;
        }
        let operator_t = self.at();
        let operator: String = if let TokenType::Operator(_) = operator_t.token_type {
            self.eat().value
        } else {
            "".to_string()
        };
        self.parse_complex_expr(
            left,
            SemiToken {
                value: operator,
                column: operator_t.position.column - 1,
                line: operator_t.position.line,
            },
        )
    }
    fn parse_math_lineal_expr(&mut self) -> ast::Node {
        let left = self.parse_math_multiplicative_expr();
        if left.is_error() {
            return left;
        }
        let token = self.at();
        if let TokenType::Operator(OperatorType::Plus | OperatorType::Minus) = token.token_type {
            if self.next().token_type == TokenType::Operator(OperatorType::Equals) {
                return left;
            }
        } else {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_math_multiplicative_expr();
        if right.is_error() {
            return right;
        }
        ast::Node::Binary(ast::NodeBinary {
            operator,
            left: left.clone().to_box(),
            right: right.to_box(),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        })
    }
    fn parse_math_multiplicative_expr(&mut self) -> ast::Node {
        let left = self.parse_math_exponetial_expr();
        if left.is_error() {
            return left;
        }
        let token = self.at();

        if let TokenType::Operator(
            OperatorType::Star | OperatorType::Division | OperatorType::Module,
        ) = token.token_type
        {
            if self.next().token_type == TokenType::Operator(OperatorType::Equals) {
                return left;
            }
        } else {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_math_exponetial_expr();
        if right.is_error() {
            return right;
        }
        ast::Node::Binary(ast::NodeBinary {
            operator,
            left: left.clone().to_box(),
            right: right.to_box(),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        })
    }
    fn parse_math_exponetial_expr(&mut self) -> ast::Node {
        let left = self.parse_literal_expr();
        if left.is_err() {
            let token = left.err().unwrap();
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Token inesperado (exponencial iz)".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            });
        }
        let left = left.ok().unwrap();
        if left.is_error() {
            return left;
        }
        let token = self.at();
        if token.token_type != TokenType::Operator(OperatorType::Power)
            || self.next().token_type == TokenType::Operator(OperatorType::Equals)
        {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_literal_expr();
        if right.is_err() {
            let token = right.err().unwrap();
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Token inesperado (exponencial de)".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            });
        }
        let right = right.ok().unwrap();
        if right.is_error() {
            return right;
        }
        ast::Node::Binary(ast::NodeBinary {
            operator,
            left: left.clone().to_box(),
            right: right.to_box(),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        })
    }
    fn parse_assignment_expr(&mut self, left: ast::Node, operator_st: SemiToken) -> ast::Node {
        let token = self.at();
        let pre_operator: String = if operator_st.line == token.position.line
            && operator_st.column == (token.position.column - 1)
            && token.token_type == TokenType::Operator(OperatorType::Equals)
        {
            self.eat();
            operator_st.value.clone()
        } else {
            "".to_string()
        };
        // operator "(?) =" operator not valid (space between operators)
        if pre_operator == "" && operator_st.value != "" {
            return left;
        }
        let operator = pre_operator.as_str();
        let right = self.parse_expr();
        if right.is_error() {
            return right;
        }
        if operator == "" {
            return ast::Node::Assignment(ast::NodeAssignment {
                identifier: left.clone().to_box(),
                value: right.to_box(),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if operator == "=" {
            return ast::Node::Binary(ast::NodeBinary {
                operator: "==".to_string(),
                left: left.clone().to_box(),
                right: right.to_box(),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if ASSIGNMENT_MODIFICATOR.contains(&operator) {
            return ast::Node::Assignment(ast::NodeAssignment {
                identifier: left.clone().to_box(),
                value: ast::Node::Binary(ast::NodeBinary {
                    operator: operator.to_string(),
                    left: left.clone().to_box(),
                    right: right.to_box(),
                    column: left.get_column(),
                    line: left.get_line(),
                    file: left.get_file(),
                })
                .to_box(),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if COMPARISON.contains(&operator) {
            return ast::Node::Binary(ast::NodeBinary {
                operator: operator.to_string(),
                left: left.clone().to_box(),
                right: right.to_box(),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        let line = self.source.lines().nth(token.position.line).unwrap();
        return ast::Node::Error(ast::NodeError {
            message: "Operador no válido para asignación".to_string(),
            column: token.position.column,
            line: token.position.line,
            meta: format!("{}\0{}\0{}", token.meta, line, token.value),
        });
    }
    fn parse_complex_expr(&mut self, left: ast::Node, operator_st: SemiToken) -> ast::Node {
        let (left, mut operator_st) = self.parse_back_unary_expr(left.clone(), operator_st);
        if left.is_error() {
            return left;
        }
        let token = self.at();
        if token.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: token.value.clone(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta,
            });
        }
        if token.token_type != TokenType::Operator(OperatorType::None) && operator_st.value == "" {
            return left;
        }
        if token.token_type != TokenType::Operator(OperatorType::None)
            && (operator_st.value == ">" || operator_st.value == "<")
        {
            let operator = operator_st.value;
            let right = self.parse_expr();
            if right.is_error() {
                return right;
            }
            return ast::Node::Binary(ast::NodeBinary {
                operator,
                left: left.clone().to_box(),
                right: right.to_box(),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if operator_st.value == "=" && token.token_type != TokenType::Operator(OperatorType::Equals)
        {
            operator_st.value = "".to_string();
            return self.parse_assignment_expr(left, operator_st);
        }
        match token.value.as_str() {
            "=" => {
                self.parse_assignment_expr(left, operator_st)
            }
            "?" | "|" | "&" => {
                self.eat();
                if operator_st.line != token.position.line
                    || operator_st.column != (token.position.column - 1)
                {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        message: "Se esperaba una expresión (c)".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                    });
                }
                operator_st.value += token.value.as_str();
                operator_st.column = token.position.column;
                if operator_st.value != "??"
                    && operator_st.value != "||"
                    && operator_st.value != "&&"
                {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        message: "Operador no válido".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                    });
                }
                if self.at().token_type == TokenType::Operator(OperatorType::Equals) {
                    return self.parse_assignment_expr(left, operator_st);
                }
                let right = self.parse_expr();
                if right.is_error() {
                    return right;
                }
                ast::Node::Binary(ast::NodeBinary {
                    operator: format!("{}", operator_st.value),
                    left: left.clone().to_box(),
                    right: right.to_box(),
                    column: left.get_column(),
                    line: left.get_line(),
                    file: left.get_file(),
                })
            }
            _ => left,
        }
    }
    fn parse_back_unary_expr(
        &mut self,
        left: ast::Node,
        mut operator_st: SemiToken,
    ) -> (ast::Node, SemiToken) {
        let token = self.at();
        if operator_st.value == "!"
            && (token.token_type != TokenType::Operator(OperatorType::Equals))
        {
            let operator = operator_st.value;
            let new_operator = if let TokenType::Operator(_) = token.token_type {
                self.eat();
                token.value
            } else {
                "".to_string()
            };
            operator_st.value = new_operator;
            operator_st.column = token.position.column;
            operator_st.line = token.position.line;
            let data = ast::Node::UnaryBack(ast::NodeUnary {
                operator,
                operand: left.to_box(),
                column: operator_st.column,
                line: operator_st.line,
                file: self.file_name.clone(),
            });
            let column = operator_st.column;
            let line = operator_st.line;
            return (
                self.parse_complex_expr(data, operator_st),
                SemiToken {
                    value: if let TokenType::Operator(_) = self.at().token_type {
                        self.eat().value
                    } else {
                        "".to_string()
                    },
                    column,
                    line,
                },
            );
        }
        if operator_st.value == ""
            && (token.token_type == TokenType::Punctuation(PunctuationType::Dot)
                || token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen)
                || token.token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketOpen)
                || token.token_type == TokenType::Punctuation(PunctuationType::DoubleDot))
        {
            let value = self.parse_call_member_expr(left);
            let token = self.at();
            return (
                value,
                SemiToken {
                    column: token.position.column,
                    line: token.position.line,
                    value: if let TokenType::Operator(_) = token.token_type {
                        self.eat();
                        token.value
                    } else {
                        "".to_string()
                    },
                },
            );
        }
        return (left, operator_st);
    }
    fn parse_call_member_expr(&mut self, object: ast::Node) -> ast::Node {
        let member = self.parse_member_expr(object);
        if member.is_error() {
            return member;
        }
        let token = self.at();
        if token.token_type == TokenType::Punctuation(PunctuationType::CircularBracketOpen) {
            return self.parse_call_expr(member);
        }
        return member;
    }
    fn parse_call_expr(&mut self, callee: ast::Node) -> ast::Node {
        let token = self.eat();
        let mut args = List::new();
        while self.not_eof()
            && !(self.at().token_type
                == TokenType::Punctuation(PunctuationType::CircularBracketClose))
        {
            let arg = self.parse_expr();
            if arg.is_error() {
                return arg;
            }
            args.push(arg);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
                self.eat();
                continue;
            }
            if comma.token_type == TokenType::Punctuation(PunctuationType::CircularBracketClose) {
                break;
            }

            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_paren = self.expect(
            TokenType::Punctuation(PunctuationType::CircularBracketClose),
            "",
        );
        if close_paren.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de cierre".to_string(),
                column: close_paren.position.column,
                line: close_paren.position.line,
                meta: close_paren.meta,
            });
        }
        let call_expr = ast::Node::Call(ast::NodeCall {
            callee: callee.to_box(),
            arguments: args,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
        let semi_token = if let TokenType::Operator(_) = self.at().token_type {
            let token = self.eat();
            SemiToken {
                column: token.position.column,
                line: token.position.line,
                value: token.value,
            }
        } else {
            SemiToken {
                column: self.at().position.column,
                line: self.at().position.line,
                value: "".to_string(),
            }
        };
        self.parse_complex_expr(call_expr, semi_token)
    }
    fn parse_member_expr(&mut self, object: ast::Node) -> ast::Node {
        let mut value = object;
        while self.at().token_type == TokenType::Punctuation(PunctuationType::Dot)
            || self.at().token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketOpen)
            || self.at().token_type == TokenType::Punctuation(PunctuationType::DoubleDot)
        {
            let operator = self.eat();
            let computed = operator.value == "[";
            let instance = operator.value == ":";
            let property: ast::Node = if computed {
                self.parse_expr()
            } else {
                if instance {
                    if self.at().token_type != TokenType::Punctuation(PunctuationType::DoubleDot) {
                        let line = self.source.lines().nth(operator.position.line).unwrap();
                        return ast::Node::Error(ast::NodeError {
                            column: operator.position.column,
                            line: operator.position.line,
                            message: "Se esperaba un identificador valido".to_string(),
                            meta: format!("{}\0{}\0{}", operator.meta, line, operator.value),
                        });
                    }
                    self.eat();
                }
                self.parse_literal_member_expr()
            };
            if property.is_error() {
                return property;
            }
            if computed {
                let close = self.expect(
                    TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
                    "",
                );
                if close.token_type == TokenType::Error {
                    return ast::Node::Error(ast::NodeError {
                        column: close.position.column,
                        line: close.position.line,
                        message: "Se esperaba un corchete de cierre".to_string(),
                        meta: close.meta,
                    });
                }
            }
            value = ast::Node::Member(ast::NodeMember {
                object: value.clone().to_box(),
                member: property.to_box(),
                computed,
                instance,
                column: value.get_column(),
                line: value.get_line(),
                file: value.get_file(),
            });
        }
        return value;
    }
    fn parse_literal_member_expr(&mut self) -> ast::Node {
        let token = self.eat();
        match token.token_type {
            TokenType::Identifier | TokenType::Keyword(_) => {
                ast::Node::Identifier(ast::NodeIdentifier {
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                    name: token.value,
                })
            }
            _ => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                ast::Node::Error(ast::NodeError {
                    column: token.position.column,
                    line: token.position.line,
                    message: "Se esperaba un identificador valido".to_string(),
                    meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                })
            }
        }
    }
    fn parse_literal_expr(&mut self) -> Result<ast::Node, Token<TokenType>> {
        let token = self.at();
        match token.token_type {
            TokenType::Identifier => Ok(ast::Node::Identifier(ast::NodeIdentifier {
                name: self.eat().value,
                column: token.position.column,
                line: token.position.line,
                file: token.meta,
            })),
            TokenType::NumberLiteral => Ok(ast::Node::Number(ast::NodeNumber {
                base: 10,
                value: self.eat().value,
                column: token.position.column,
                line: token.position.line,
                file: token.meta,
            })),
            TokenType::Number => {
                self.eat();
                let data = token.value.split("$").collect::<Vec<_>>()[1];
                let base_value = data.split("~").collect::<Vec<_>>();
                let base = base_value[0].parse::<u8>().unwrap();
                let value = base_value[1].to_string();
                return Ok(ast::Node::Number(ast::NodeNumber {
                    base,
                    value,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                }));
            }
            TokenType::StringLiteral => Ok(ast::Node::String(ast::NodeString {
                value: List::from_vec(vec![ast::StringData::Str(self.eat().value)]),
                column: token.position.column,
                line: token.position.line,
                file: token.meta,
            })),
            TokenType::String => {
                self.eat();
                let line = self.source.lines().nth(token.position.line).unwrap();
                let node = string::complex_string(token, line);
                if node.is_err() {
                    return Ok(ast::Node::Error(node.err().unwrap()));
                }
                Ok(ast::Node::String(node.ok().unwrap()))
            }
            TokenType::Punctuation(PunctuationType::RegularBracketOpen) => {
                Ok(self.parse_object_expr())
            }
            TokenType::Punctuation(PunctuationType::CircularBracketOpen) => {
                self.eat();
                let expr = self.parse_expr();
                let close_paren = self.expect(
                    TokenType::Punctuation(PunctuationType::CircularBracketClose),
                    "",
                );
                if close_paren.token_type == TokenType::Error {
                    return Ok(ast::Node::Error(ast::NodeError {
                        message: "Se esperaba un paréntesis de cierre".to_string(),
                        column: close_paren.position.column,
                        line: close_paren.position.line,
                        meta: close_paren.meta,
                    }));
                }
                return Ok(expr);
            }
            TokenType::Punctuation(PunctuationType::QuadrateBracketOpen) => {
                Ok(self.parse_array_expr())
            }
            TokenType::Operator(
                OperatorType::Minus
                | OperatorType::Plus
                | OperatorType::Negative
                | OperatorType::Not
                | OperatorType::And
                | OperatorType::QuestionMark,
            ) => {
                let expr = self.parse_literal_expr();
                if expr.is_err() {
                    return Err(token);
                }
                let expr = expr.ok().unwrap();
                return Ok(ast::Node::UnaryFront(ast::NodeUnary {
                    operator: token.value,
                    operand: expr.to_box(),
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                }));
            }
            TokenType::Keyword(
                KeywordsType::Mientras
                | KeywordsType::Hacer
                | KeywordsType::Si
                | KeywordsType::Funcion
                | KeywordsType::Intentar,
            ) => Ok(self.parse_keyword_value(false, false)),
            _ => Err(token),
        }
    }
    fn parse_object_expr(&mut self) -> ast::Node {
        let open_brace = self.eat();
        let mut properties = List::new();

        while self.not_eof()
            && !(self.at().token_type
                == TokenType::Punctuation(PunctuationType::RegularBracketClose))
        {
            let property = self.parse_object_property();
            if property.is_err() {
                return ast::Node::Error(property.err().unwrap());
            }
            let property = property.ok().unwrap();
            properties.push(property);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
                self.eat();
                continue;
            }
            if comma.token_type == TokenType::Punctuation(PunctuationType::RegularBracketClose) {
                break;
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_brace = self.expect(
            TokenType::Punctuation(PunctuationType::RegularBracketClose),
            "",
        );
        if close_brace.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una llave de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: close_brace.meta,
            });
        }
        ast::Node::Object(ast::NodeObject {
            properties,
            column: open_brace.position.column,
            line: open_brace.position.line,
            file: open_brace.meta,
        })
    }
    fn parse_object_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
        let token = self.eat();
        match token.token_type {
            TokenType::StringLiteral => {
                let key = token.value;
                let colon = self.expect(
                    TokenType::Punctuation(PunctuationType::DoubleDot),
                    "Se esperaba dos puntos",
                );
                if colon.token_type == TokenType::Error {
                    return Err(ast::NodeError {
                        message: colon.value,
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: colon.meta,
                    });
                }
                let value = self.parse_expr();
                return Ok(ast::NodeProperty::Property(key, value));
            }
            TokenType::Identifier => {
                let key = &token.value;
                let colon = self.eat();
                if colon.token_type == TokenType::Error {
                    return Err(ast::NodeError {
                        message: "Se esperaba dos puntos".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: colon.meta,
                    });
                }
                // the key is a variable name and value is an identifier
                if colon.token_type == TokenType::Punctuation(PunctuationType::Comma)
                    || colon.token_type
                        == TokenType::Punctuation(PunctuationType::RegularBracketClose)
                {
                    self.index -= 1;
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
                if colon.token_type != TokenType::Punctuation(PunctuationType::DoubleDot) {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba dos puntos".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                let value = self.parse_expr();
                return Ok(ast::NodeProperty::Property(key.clone(), value));
            }
            TokenType::Punctuation(p) => {
                if p == PunctuationType::QuadrateBracketOpen {
                    let expr = self.parse_expr();
                    let close_bracket = self.expect(
                        TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
                        "Se esperaba un corchete de cierre",
                    );
                    if close_bracket.token_type == TokenType::Error {
                        let line = self
                            .source
                            .lines()
                            .nth(close_bracket.position.line)
                            .unwrap();
                        let meta =
                            format!("{}\0{}\0{}", close_bracket.meta, line, &close_bracket.value);
                        return Err(ast::NodeError {
                            message: close_bracket.value,
                            column: close_bracket.position.column,
                            line: close_bracket.position.line,
                            meta,
                        });
                    }
                    let key = expr;
                    let colon = self.expect(
                        TokenType::Punctuation(PunctuationType::DoubleDot),
                        "Se esperaba dos puntos",
                    );
                    if colon.token_type == TokenType::Error {
                        return Err(ast::NodeError {
                            message: colon.value,
                            column: colon.position.column,
                            line: colon.position.line,
                            meta: colon.meta,
                        });
                    }
                    let value = self.parse_expr();
                    return Ok(ast::NodeProperty::Dynamic(key, value));
                }
                if p == PunctuationType::Dot {
                    let dot = self.expect(
                        TokenType::Punctuation(PunctuationType::Dot),
                        "Se esperaba un punto",
                    );
                    if dot.token_type == TokenType::Error {
                        return Err(ast::NodeError {
                            message: dot.value,
                            column: dot.position.column,
                            line: dot.position.line,
                            meta: dot.meta,
                        });
                    }
                    let data = self.parse_expr();
                    if data.is_error() {
                        return Err(data.get_error().unwrap().clone());
                    }
                    return Ok(ast::NodeProperty::Iterable(data));
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
    fn parse_array_expr(&mut self) -> ast::Node {
        let open_bracket = self.eat();
        let mut elements = List::new();

        while self.not_eof()
            && !(self.at().token_type
                == TokenType::Punctuation(PunctuationType::QuadrateBracketClose))
        {
            let element = self.parse_array_property();
            if element.is_err() {
                return ast::Node::Error(element.err().unwrap());
            }
            let property = element.ok().unwrap();
            elements.push(property);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation(PunctuationType::Comma) {
                self.eat();
                continue;
            }
            if comma.token_type == TokenType::Punctuation(PunctuationType::QuadrateBracketClose) {
                break;
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_brace = self.expect(
            TokenType::Punctuation(PunctuationType::QuadrateBracketClose),
            "Se esperaba un corchete de cierre",
        );
        if close_brace.token_type == TokenType::Error {
            return ast::Node::Error(ast::NodeError {
                message: close_brace.value,
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: close_brace.meta,
            });
        }
        ast::Node::Array(ast::NodeArray {
            elements,
            column: open_bracket.position.column,
            line: open_bracket.position.line,
            file: open_bracket.meta,
        })
    }
    fn parse_array_property(&mut self) -> Result<ast::NodeProperty, ast::NodeError> {
        let token = self.at();
        match token.token_type {
            TokenType::Punctuation(p) => {
                if p == PunctuationType::Dot {
                    self.eat();
                    let dot = self.expect(
                        TokenType::Punctuation(PunctuationType::Dot),
                        "Se esperaba un punto",
                    );
                    if dot.token_type == TokenType::Error {
                        return Err(ast::NodeError {
                            message: dot.value,
                            column: dot.position.column,
                            line: dot.position.line,
                            meta: dot.meta,
                        });
                    }
                    let data = self.parse_expr();
                    if data.is_error() {
                        return Err(data.get_error().unwrap().clone());
                    }
                    return Ok(ast::NodeProperty::Iterable(data));
                }
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Err(ast::NodeError {
                    message: "Se esperaba un valor para la lista".to_string(),
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                });
            }
            _ => {
                let element = self.parse_expr();
                return Ok(ast::NodeProperty::Indexable(element));
            }
        }
    }
}
