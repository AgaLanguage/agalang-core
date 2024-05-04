pub mod ast;
pub mod string;
use util::Token;

use crate::{
    internal::errors::ErrorTypes,
    util::{split_meta, to_cyan},
};

use super::lexer::TokenType;

const ASSIGNMENT_PREV: [&str; 8] = ["+", "-", "*", "/", "%", "&&", "||", "??"];
const COMPARISON: [&str; 4] = ["=", "!", "<", ">"];
const MISSING_TOKEN: &str = "\x1b[81mToken desaparecido\x1b[0m";

struct SemiToken {
    value: String,
    column: usize,
    line: usize,
}

fn node_error(node: &ast::Node) -> ErrorTypes {
    let line: usize;
    let column_node: usize;
    let meta: &str;
    let message: &str;
    match node {
        ast::Node::Error(error) => {
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
        }
        _ => {
            line = 1;
            column_node = 1;
            meta = "<indeterminado>";
            message = "A ocurrido un error";
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
    fn at(&self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        if token.is_none() {
            let position = self.prev().position;
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position,
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
    fn next(&self) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index + 1);
        if token.is_none() {
            let position = self.at().position;
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: "Se esperaba un token".to_string(),
                position,
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
    fn expect(&mut self, token_type: TokenType, err: &str) -> util::Token<TokenType> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        if token.is_none() {
            let position = self.prev().position;
            return util::Token::<TokenType> {
                token_type: TokenType::Error,
                value: err.to_string(),
                position,
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
    pub fn produce_ast(&mut self) -> ast::Node {
        let mut program = ast::NodeProgram {
            body: Vec::new(),
            column: 0,
            line: 0,
            file: self.file_name.clone(),
        };
        // let functions: Vec<ast::Node> = Vec::new();
        //let code: Vec<Box<dyn ast::Node>> = Vec::new();

        while self.not_eof() {
            let stmt = self.parse_stmt(true, false, false, false);
            if let Some(stmt) = stmt {
                match stmt {
                    ast::Node::Error(error) => {
                        let node = &ast::Node::Error(error);
                        crate::internal::errors::throw_error(
                            crate::internal::errors::ErrorNames::SyntaxError,
                            node_error(node),
                        );
                        break;
                    }
                    _ => {
                        program.body.push(stmt);
                    }
                }
            }
        }
        ast::Node::Program(program)
    }
    fn parse_stmt(
        &mut self,
        is_global_scope: bool,
        is_function: bool,
        is_class_decl: bool,
        is_loop: bool,
    ) -> Option<ast::Node> {
        let token = self.at();
        match token.token_type {
            TokenType::EOF => {
                self.eat();
                None
            }
            TokenType::Error => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                return Some(ast::Node::Error(ast::NodeError {
                    message: token.value,
                    column: token.position.column,
                    line: token.position.line,
                    meta: format!(
                        "{}\0{}\0{}",
                        token.meta,
                        line,
                        " ".repeat(token.position.column)
                    ),
                }));
            }
            TokenType::Keyword => match token.value.as_str() {
                "def" | "const" => Some(self.parse_var_decl()),
                "mien" => Some(self.parse_while_decl(is_function)),
                _ => {
                    self.eat();
                    None
                }
            },
            _ => Some(self.parse_stmt_expr()),
        }
    }
    fn parse_do_while_decl(&mut self, is_function: bool) -> ast::Node {
        let token = self.eat(); // hacer
        let block = self.parse_block_expr(is_function, true);
        if block.is_err() {
            return block.err().unwrap();
        }
        let body = block.ok().unwrap();
        let while_token = self.expect(TokenType::Keyword, "Se esperaba la palabra clave 'mien'");
        if while_token.token_type == TokenType::Error || while_token.value != "mien" {
            let line = self.source.lines().nth(while_token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba la palabra clave 'mien'".to_string(),
                column: while_token.position.column,
                line: while_token.position.line,
                meta: format!("{}\0{}\0{}", while_token.meta, line, while_token.value),
            });
        }
        let condition = self.parse_expr();
        if condition.is_error() {
            return condition;
        }
        let semicolon = self.expect(TokenType::Punctuation, "");
        if semicolon.token_type == TokenType::Error || semicolon.value != ";" {
            let line = self.source.lines().nth(semicolon.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: format!("{}\0{}\0{}", semicolon.meta, line, semicolon.value),
            });
        }
        return ast::Node::DoWhile(ast::NodeWhile {
            condition: Box::new(condition),
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
            return block.err().unwrap();
        }
        let body = block.ok().unwrap();
        return ast::Node::While(ast::NodeWhile {
            condition: Box::new(condition),
            body,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
    }
    fn parse_block_expr(&mut self, is_function: bool, is_loop: bool) -> Result<Vec<ast::Node>, ast::Node> {
        let open_brace = self.at();
        if open_brace.token_type == TokenType::Error {
            let line = self.source.lines().nth(open_brace.position.line).unwrap();
            return Err(ast::Node::Error(ast::NodeError {
                message: "Se esperaba un bloque".to_string(),
                column: open_brace.position.column,
                line: open_brace.position.line,
                meta: format!("{}\0{}\0{}", open_brace.meta, line, open_brace.value),
            }));
        }
        if self.at().token_type != TokenType::Punctuation || self.at().value != "{" {
            let expr = self.parse_stmt(false, is_function, false, is_loop);
            if expr.is_none() {
                let line = self.source.lines().nth(open_brace.position.line).unwrap();
                return Err(ast::Node::Error(ast::NodeError {
                    message: "Se esperaba un bloque".to_string(),
                    column: open_brace.position.column,
                    line: open_brace.position.line,
                    meta: format!("{}\0{}\0{}", open_brace.meta, line, open_brace.value),
                }));
            }
            let expr = expr.unwrap();
            if expr.is_error() {
                return Err(expr);
            }
            return Ok(vec![expr]);
        }
        self.eat();
        let mut body: Vec<ast::Node> = Vec::new();
        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "}")
        {
            let stmt = self.parse_stmt(false, false, false, false);
            if stmt.is_none() {
                break;
            }
            let stmt = stmt.unwrap();
            if stmt.is_error() {
                return Err(stmt);
            }
            body.push(stmt);
        }
        let close_brace = self.expect(TokenType::Punctuation, "");
        if close_brace.token_type == TokenType::Error || close_brace.value != "}" {
            let line = self.source.lines().nth(close_brace.position.line).unwrap();
            return Err(ast::Node::Error(ast::NodeError {
                message: "Se esperaba un corchete de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: format!("{}\0{}\0{}", close_brace.meta, line, close_brace.value),
            }));
        }
        Ok(body)
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
        if equals_semicolon.token_type == TokenType::Punctuation && equals_semicolon.value == ";" {
            return ast::Node::VarDecl(ast::NodeVarDecl {
                name: identifier.value.clone(),
                value: None,
                is_const,
                column: identifier.position.column,
                line: identifier.position.line,
                file: identifier.meta,
            });
        }
        if equals_semicolon.token_type != TokenType::Operator || equals_semicolon.value != "=" {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
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
        let semicolon = self.expect(TokenType::Punctuation, "");
        if semi_token.line == semicolon.position.line {
            semi_token.value += " "
                .repeat(semicolon.position.column - semi_token.column)
                .as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.column += 1;
        if semicolon.token_type == TokenType::Error || semicolon.value != ";" {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
                column: semi_token.column,
                line: semi_token.line,
                meta,
            });
        }
        return ast::Node::VarDecl(ast::NodeVarDecl {
            name: identifier.value.clone(),
            value: Some(Box::new(value)),
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
        let semicolon = self.expect(TokenType::Punctuation, "");
        if semicolon.token_type == TokenType::Error || semicolon.value != ";" {
            let line = self.source.lines().nth(semicolon.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
                column: semicolon.position.column,
                line: semicolon.position.line,
                meta: format!("{}\0{}\0{}", semicolon.meta, line, semicolon.value),
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
        let operator: String = if operator_t.token_type == TokenType::Operator {
            self.eat().value
        } else {
            "".to_string()
        };
        self.parse_complex_expr(
            left,
            SemiToken {
                value: operator,
                column: operator_t.position.column,
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
        if token.token_type != TokenType::Operator
            || (token.value != "+" && token.value != "-")
            || self.next().token_type == TokenType::Operator
        {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_math_multiplicative_expr();
        if right.is_error() {
            return right;
        }
        ast::Node::Binary(ast::NodeBinary {
            operator,
            left: Box::new(left.clone()),
            right: Box::new(right),
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
        if token.token_type != TokenType::Operator
            || (token.value != "*" && token.value != "/" && token.value != "%")
            || self.next().token_type == TokenType::Operator
        {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_math_exponetial_expr();
        if right.is_error() {
            return right;
        }
        ast::Node::Binary(ast::NodeBinary {
            operator,
            left: Box::new(left.clone()),
            right: Box::new(right),
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
                message: "Token inesperado".to_string(),
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
        if token.token_type != TokenType::Operator
            || token.value != "^"
            || self.next().token_type == TokenType::Operator
        {
            return left;
        }
        let operator = self.eat().value;
        let right = self.parse_literal_expr();
        if right.is_err() {
            let token = right.err().unwrap();
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Token inesperado".to_string(),
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
            left: Box::new(left.clone()),
            right: Box::new(right),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        })
    }
    fn parse_assignment_expr(&mut self, left: ast::Node, operator_st: SemiToken) -> ast::Node {
        let token = self.prev();
        let operator: &str = &operator_st.value;
        if operator_st.line != token.position.line
            || operator_st.column != (token.position.column - 1)
        {
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            });
        }
        let right = self.parse_expr();
        if right.is_error() {
            return right;
        }
        if operator == "" {
            return ast::Node::Assignment(ast::NodeAssignment {
                identifier: Box::new(left.clone()),
                value: Box::new(right),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if operator == "=" {
            return ast::Node::Binary(ast::NodeBinary {
                operator: "==".to_string(),
                left: Box::new(left.clone()),
                right: Box::new(right),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if ASSIGNMENT_PREV.contains(&operator) {
            return ast::Node::Assignment(ast::NodeAssignment {
                identifier: Box::new(left.clone()),
                value: Box::new(ast::Node::Binary(ast::NodeBinary {
                    operator: operator.to_string(),
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                    column: left.get_column(),
                    line: left.get_line(),
                    file: left.get_file(),
                })),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        if COMPARISON.contains(&operator) {
            return ast::Node::Binary(ast::NodeBinary {
                operator: format!("{}=", operator),
                left: Box::new(left.clone()),
                right: Box::new(right),
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
            let line = self.source.lines().nth(token.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: token.value.clone(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            });
        }
        if token.token_type != TokenType::Operator && operator_st.value == "" {
            return left;
        }
        if token.token_type != TokenType::Operator && (operator_st.value == ">" || operator_st.value == "<") {
            let operator = operator_st.value;
            let right = self.parse_expr();
            if right.is_error() {
                return right;
            }
            return ast::Node::Binary(ast::NodeBinary {
                operator,
                left: Box::new(left.clone()),
                right: Box::new(right),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            });
        }
        self.eat();
        match token.value.as_str() {
            "=" => self.parse_assignment_expr(left, operator_st),
            "?" | "|" | "&" => {
                if operator_st.line != token.position.line
                    || operator_st.column != (token.position.column - 1)
                {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        message: "Se esperaba una expresión".to_string(),
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
                if self.at().token_type == TokenType::Operator && self.at().value == "=" {
                    return self.parse_assignment_expr(left, operator_st);
                }
                let right = self.parse_expr();
                if right.is_error() {
                    return right;
                }
                ast::Node::Binary(ast::NodeBinary {
                    operator: format!("{}", operator_st.value),
                    left: Box::new(left.clone()),
                    right: Box::new(right),
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
            && (token.token_type != TokenType::Operator || token.value != "=")
        {
            let operator = operator_st.value;
            let new_operator = if token.token_type == TokenType::Operator {
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
                operand: Box::new(left),
                column: operator_st.column,
                line: operator_st.line,
                file: self.file_name.clone(),
            });
            let column = operator_st.column;
            let line = operator_st.line;
            return (
                self.parse_complex_expr(data, operator_st),
                SemiToken {
                    value: if self.at().token_type == TokenType::Operator {
                        self.eat();
                        self.at().value
                    } else {
                        "".to_string()
                    },
                    column,
                    line,
                },
            );
        }
        if operator_st.value == ""
            && (token.token_type == TokenType::Punctuation)
            && (token.value == "." || token.value == "[" || token.value == "(")
        {
            let value = self.parse_call_member_expr(left);
            return (value, operator_st);
        }
        return (left, operator_st);
    }
    fn parse_call_member_expr(&mut self, object: ast::Node) -> ast::Node {
        let member = self.parse_member_expr(object);
        if member.is_error() {
            return member;
        }
        let token = self.at();
        if token.token_type == TokenType::Punctuation && token.value == "(" {
            return self.parse_call_expr(member);
        }
        return member;
    }
    fn parse_call_expr(&mut self, callee: ast::Node) -> ast::Node {
        let token = self.eat();
        let mut args: Vec<ast::Node> = Vec::new();
        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == ")")
        {
            let arg = self.parse_expr();
            if arg.is_error() {
                return arg;
            }
            args.push(arg);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation {
                if comma.value == "," {
                    self.eat();
                    continue;
                }
                if comma.value == ")" {
                    break;
                }
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_paren = self.expect(TokenType::Punctuation, "");
        if close_paren.token_type == TokenType::Error || close_paren.value != ")" {
            let line = self.source.lines().nth(close_paren.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un paréntesis de cierre".to_string(),
                column: close_paren.position.column,
                line: close_paren.position.line,
                meta: format!("{}\0{}\0{}", close_paren.meta, line, close_paren.value),
            });
        }
        let call_expr = ast::Node::Call(ast::NodeCall {
            callee: Box::new(callee),
            arguments: args,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        });
        let semi_token = if self.at().token_type == TokenType::Operator {
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
        while (self.at().token_type == TokenType::Punctuation)
            && (self.at().value == "." || self.at().value == "[")
        {
            let operator = self.eat();
            let computed = operator.value == "[";
            let property: ast::Node = if operator.value == "." {
                self.parse_literal_member_expr()
            } else {
                self.parse_expr()
            };
            if property.is_error() {
                return property;
            }
            if computed {
                let close = self.expect(TokenType::Punctuation, "");
                if close.token_type == TokenType::Error || close.value != "]" {
                    let line = self.source.lines().nth(close.position.line).unwrap();
                    return ast::Node::Error(ast::NodeError {
                        column: close.position.column,
                        line: close.position.line,
                        message: "Se esperaba un corchete de cierre".to_string(),
                        meta: format!("{}\0{}\0{}", close.meta, line, close.value),
                    });
                }
            }
            value = ast::Node::Member(ast::NodeMember {
                object: Box::new(value.clone()),
                member: Box::new(property),
                computed,
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
            TokenType::Identifier | TokenType::Keyword => {
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
        let token = self.eat();
        match token.token_type {
            TokenType::Identifier => {
                return Ok(ast::Node::Identifier(ast::NodeIdentifier {
                    name: token.value.clone(),
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                }));
            }
            TokenType::NumberLiteral => {
                return Ok(ast::Node::Number(ast::NodeNumber {
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
                return Ok(ast::Node::Number(ast::NodeNumber {
                    base,
                    value,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                }));
            }
            TokenType::StringLiteral => {
                return Ok(ast::Node::String(ast::NodeString {
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
                        return Ok(ast::Node::String(node));
                    }
                    Err(error) => {
                        return Ok(ast::Node::Error(error));
                    }
                }
            }
            TokenType::Punctuation => match token.value.as_str() {
                "{" => {
                    let expr = self.parse_object_expr();
                    return Ok(expr);
                }
                "(" => {
                    let expr = self.parse_expr();
                    let close_paren = self.expect(TokenType::Punctuation, "");
                    if close_paren.token_type == TokenType::Error || close_paren.value != ")" {
                        let line = self.source.lines().nth(close_paren.position.line).unwrap();
                        return Ok(ast::Node::Error(ast::NodeError {
                            message: "Se esperaba un paréntesis de cierre".to_string(),
                            column: close_paren.position.column,
                            line: close_paren.position.line,
                            meta: format!("{}\0{}\0{}", close_paren.meta, line, close_paren.value),
                        }));
                    }
                    return Ok(expr);
                }
                "[" => {
                    let expr = self.parse_array_expr();
                    return Ok(expr);
                }
                _ => Err(token),
            },
            TokenType::Operator => match token.value.as_str() {
                "-" | "+" | "~" | "!" | "&" | "?" => {
                    let expr = self.parse_literal_expr();
                    if expr.is_err() {
                        return Err(token);
                    }
                    let expr = expr.ok().unwrap();
                    return Ok(ast::Node::UnaryFront(ast::NodeUnary {
                        operator: token.value,
                        operand: Box::new(expr),
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    }));
                }
                _ => Err(token),
            },
            _ => Err(token),
        }
    }
    fn parse_object_expr(&mut self) -> ast::Node {
        let open_brace = self.prev();
        let mut properties: Vec<ast::NodeProperty> = Vec::new();

        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "}")
        {
            let property = self.parse_object_property();
            if property.is_err() {
                return ast::Node::Error(property.err().unwrap());
            }
            let property = property.ok().unwrap();
            properties.push(property);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation {
                if comma.value == "," {
                    self.eat();
                    continue;
                }
                if comma.value == "}" {
                    break;
                }
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_brace = self.expect(TokenType::Punctuation, "");
        if close_brace.token_type == TokenType::Error || close_brace.value != "}" {
            let line = self.source.lines().nth(close_brace.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una llave de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: format!("{}\0{}\0{}", close_brace.meta, line, close_brace.value),
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
                return Ok(ast::NodeProperty::Property(key, value));
            }
            TokenType::Identifier => {
                let key = &token.value;
                let colon = self.expect(TokenType::Punctuation, "");
                if colon.token_type == TokenType::Error {
                    let line = self.source.lines().nth(colon.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba dos puntos".to_string(),
                        column: colon.position.column,
                        line: colon.position.line,
                        meta: format!("{}\0{}\0{}", colon.meta, line, colon.value),
                    });
                }
                // the key is a variable name and value is an identifier
                if colon.value == "," || colon.value == "}" {
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
                if colon.value != ":" {
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
            TokenType::Punctuation => {
                if token.value == "[" {
                    let expr = self.parse_expr();
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
                    return Ok(ast::NodeProperty::Iterable(ast::NodeIdentifier {
                        name: iterable.value,
                        column: iterable.position.column,
                        line: iterable.position.line,
                        file: iterable.meta,
                    }));
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
        let open_bracket = self.prev();
        let mut elements: Vec<ast::NodeProperty> = Vec::new();

        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "]")
        {
            let element = self.parse_array_property();
            if element.is_err() {
                return ast::Node::Error(element.err().unwrap());
            }
            let property = element.ok().unwrap();
            elements.push(property);
            let comma = self.at();
            if comma.token_type == TokenType::Punctuation {
                if comma.value == "," {
                    self.eat();
                    continue;
                }
                if comma.value == "]" {
                    break;
                }
            }
            let line = self.source.lines().nth(comma.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            });
        }
        let close_brace = self.expect(TokenType::Punctuation, "");
        if close_brace.token_type == TokenType::Error || close_brace.value != "]" {
            let line = self.source.lines().nth(close_brace.position.line).unwrap();
            return ast::Node::Error(ast::NodeError {
                message: "Se esperaba un corchete de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: format!("{}\0{}\0{}", close_brace.meta, line, close_brace.value),
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
            TokenType::Punctuation => {
                if token.value == "." {
                    self.eat();
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
                    return Ok(ast::NodeProperty::Iterable(ast::NodeIdentifier {
                        name: iterable.value,
                        column: iterable.position.column,
                        line: iterable.position.line,
                        file: iterable.meta,
                    }));
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
