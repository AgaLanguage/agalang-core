pub mod ast;
pub mod string;
use util::Token;

use crate::{
    internal::errors::ErrorTypes,
    util::{split_meta, to_cyan},
};

use super::lexer::TokenType;

const ASSIGNMENT_PREV: [&str; 10] = ["+", "-", "*", "/", "%", "&&", "||", "!", "<", ">"];
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
    pub fn produce_ast(&mut self, is_function: bool) -> ast::Node {
        let mut program = ast::NodeProgram {
            body: Vec::new(),
            column: 0,
            line: 0,
            file: self.file_name.clone(),
        };
        // let functions: Vec<ast::Node> = Vec::new();
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
        is_function: bool,
        is_class_decl: bool,
        is_global_scope: bool,
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
                "def" | "const" => self.parse_var_decl(),
                _ => {
                    self.eat();
                    None
                }
            },
            _ => self.parse_stmt_expr(),
        }
    }
    fn parse_var_decl(&mut self) -> Option<ast::Node> {
        let token = self.eat();
        let is_const = token.value == "const";
        let mut semi_token = SemiToken {
            value: token.value,
            column: token.position.column,
            line: token.position.line,
        };

        let identifier = self.expect(TokenType::Identifier, "Se esperaba un identificador");
        if semi_token.line == identifier.position.line {
            semi_token.value += " ".repeat(identifier.position.column - semi_token.column).as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.line = identifier.position.line;
        semi_token.column = identifier.position.column;
        if identifier.token_type == TokenType::Error {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return Some(ast::Node::Error(ast::NodeError {
                message: identifier.value,
                column: semi_token.column,
                line: semi_token.line,
                meta,
            }));
        }
        semi_token.value += identifier.value.as_str();
        
        let equals_semicolon = self.eat();
        if semi_token.line == equals_semicolon.position.line {
            semi_token.value += " ".repeat(equals_semicolon.position.column - semi_token.column).as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.line = equals_semicolon.position.line;
        semi_token.column = equals_semicolon.position.column;
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
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
                column: semi_token.column,
                line: semi_token.line,
                meta,
            }));
        }
        semi_token.value += equals_semicolon.value.as_str();

        let value = self.parse_expr();
        if value.is_none() {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: semi_token.column,
                line: semi_token.line,
                meta,
            }));
        }
        let value = value.unwrap();
        if semi_token.line == value.get_line() {
            semi_token.value += " ".repeat(value.get_column() - semi_token.column).as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.column = value.get_column();
        if value.is_error() {
            return Some(value);
        }
        let semicolon = self.expect(TokenType::Punctuation, "");
        if semi_token.line == semicolon.position.line {
            semi_token.value += " ".repeat(semicolon.position.column - semi_token.column).as_str();
        } else {
            semi_token.value = "".to_string();
        };
        semi_token.column += 1;
        if semicolon.token_type == TokenType::Error || semicolon.value != ";" {
            let line = self.source.lines().nth(semi_token.line).unwrap();
            let meta = format!("{}\0{}\0{}", self.file_name, line, semi_token.value);
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba un punto y coma".to_string(),
                column: semi_token.column,
                line: semi_token.line,
                meta,
            }));
        }
        return Some(ast::Node::VarDecl(ast::NodeVarDecl {
            name: identifier.value.clone(),
            value: Some(Box::new(value)),
            is_const,
            column: token.position.column,
            line: token.position.line,
            file: token.meta,
        }));
    }
    fn parse_stmt_expr(&mut self) -> Option<ast::Node> {
        let token = self.at();
        let node = self.parse_expr();
        if node.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        let node = node.unwrap();
        if node.is_error() {
            return Some(node);
        }
        let token = self.at();
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
        Some(node)
    }
    fn parse_expr(&mut self) -> Option<ast::Node> {
        let left: Option<ast::Node> = self.parse_math_lineal_expr();
        if left.is_none() {
            return left;
        }
        let left = left.unwrap();
        if left.is_error() {
            return Some(left);
        }
        let operator_t = self.at();
        let operator: String = if operator_t.token_type == TokenType::Operator {
            self.eat().value
        } else {
            "".to_string()
        };
        let expr = self.parse_complex_expr(
            left,
            SemiToken {
                value: operator,
                column: operator_t.position.column,
                line: operator_t.position.line,
            },
        );
        if expr.is_none() {
            return expr;
        }
        let expr = expr.unwrap();
        Some(expr)
    }
    fn parse_math_lineal_expr(&mut self) -> Option<ast::Node> {
        let left = self.parse_math_multiplicative_expr();
        if left.is_none() {
            return left;
        }
        let left = left.unwrap();
        if left.is_error() {
            return Some(left);
        }
        let token = self.at();
        if token.token_type != TokenType::Operator
            || (token.value != "+" && token.value != "-")
            || self.next().token_type == TokenType::Operator
        {
            return Some(left);
        }
        let operator = self.eat().value;
        let right = self.parse_math_multiplicative_expr();
        if right.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        let right = right.unwrap();
        if right.is_error() {
            return Some(right);
        }
        Some(ast::Node::Binary(ast::NodeBinary {
            operator,
            left: Box::new(left.clone()),
            right: Box::new(right),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        }))
    }
    fn parse_math_multiplicative_expr(&mut self) -> Option<ast::Node> {
        let left = self.parse_math_exponetial_expr();
        if left.is_none() {
            return left;
        }
        let left = left.unwrap();
        if left.is_error() {
            return Some(left);
        }
        let token = self.at();
        if token.token_type != TokenType::Operator
            || (token.value != "*" && token.value != "/" && token.value != "%")
            || self.next().token_type == TokenType::Operator
        {
            return Some(left);
        }
        let operator = self.eat().value;
        let right = self.parse_math_exponetial_expr();
        if right.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        let right = right.unwrap();
        if right.is_error() {
            return Some(right);
        }
        Some(ast::Node::Binary(ast::NodeBinary {
            operator,
            left: Box::new(left.clone()),
            right: Box::new(right),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        }))
    }
    fn parse_math_exponetial_expr(&mut self) -> Option<ast::Node> {
        let left = self.parse_literal_expr();
        if left.is_err() {
            let token = left.err().unwrap();
            let line = self.source.lines().nth(token.position.line).unwrap();
            return Some(ast::Node::Error(ast::NodeError {
                message: "Token inesperado".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            }));
        }
        let left = left.ok().unwrap();
        if left.is_none() {
            return left;
        }
        let left = left.unwrap();
        if left.is_error() {
            return Some(left);
        }
        let token = self.at();
        if token.token_type != TokenType::Operator
            || token.value != "^"
            || self.next().token_type == TokenType::Operator
        {
            return Some(left);
        }
        let operator = self.eat().value;
        let right = self.parse_literal_expr();
        if right.is_err() {
            let token = right.err().unwrap();
            let line = self.source.lines().nth(token.position.line).unwrap();
            return Some(ast::Node::Error(ast::NodeError {
                message: "Token inesperado".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: format!("{}\0{}\0{}", token.meta, line, token.value),
            }));
        }
        let right = right.ok().unwrap();
        if right.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        let right = right.unwrap();
        if right.is_error() {
            return Some(right);
        }
        Some(ast::Node::Binary(ast::NodeBinary {
            operator,
            left: Box::new(left.clone()),
            right: Box::new(right),
            column: left.get_column(),
            line: left.get_line(),
            file: left.get_file(),
        }))
    }
    fn parse_assignment_expr(
        &mut self,
        left: ast::Node,
        mut operator_st: SemiToken,
    ) -> Option<ast::Node> {
        let token = self.prev();
        let operator: &str = &operator_st.value;
        let right = self.parse_expr();
        if right.is_none() {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        let right = right.unwrap();
        if right.is_error() {
            return Some(right);
        }
        if operator == "" {
            return Some(ast::Node::Assignment(ast::NodeAssignment {
                identifier: Box::new(left.clone()),
                value: Box::new(right),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            }));
        }
        if operator_st.line != token.position.line
            || operator_st.column != (token.position.column + 1)
        {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una expresión".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        if operator == "=" {
            return Some(ast::Node::Binary(ast::NodeBinary {
                operator: "==".to_string(),
                left: Box::new(left.clone()),
                right: Box::new(right),
                column: left.get_column(),
                line: left.get_line(),
                file: left.get_file(),
            }));
        }
        if !ASSIGNMENT_PREV.contains(&operator) {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Operador no válido para asignación".to_string(),
                column: token.position.column,
                line: token.position.line,
                meta: token.meta.clone(),
            }));
        }
        Some(ast::Node::Assignment(ast::NodeAssignment {
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
        }))
    }
    fn parse_complex_expr(&mut self, left: ast::Node, operator_st: SemiToken) -> Option<ast::Node> {
        let token = self.at();
        if token.token_type != TokenType::Operator {
            return Some(left);
        }
        self.eat();
        match token.value.as_str() {
            "=" => self.parse_assignment_expr(left, operator_st),
            "?" => Some(ast::Node::UnaryBack(ast::NodeUnary {
                operator: token.value,
                operand: Box::new(left),
                column: token.position.column,
                line: token.position.line,
                file: token.meta,
            })),
            _ => return Some(left),
        }
    }
    fn parse_literal_expr(&mut self) -> Result<Option<ast::Node>, Token<TokenType>> {
        let token = self.eat();
        match token.token_type {
            TokenType::Identifier => {
                return Ok(Some(ast::Node::Identifier(ast::NodeIdentifier {
                    name: token.value.clone(),
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                })));
            }
            TokenType::NumberLiteral => {
                return Ok(Some(ast::Node::Number(ast::NodeNumber {
                    base: 10,
                    value: token.value.clone(),
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                })));
            }
            TokenType::Number => {
                let data = token.value.split("$").collect::<Vec<&str>>()[1];
                let base_value = data.split("~").collect::<Vec<&str>>();
                let base = base_value[0].parse::<i8>().unwrap();
                let value = base_value[1].to_string();
                return Ok(Some(ast::Node::Number(ast::NodeNumber {
                    base,
                    value,
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                })));
            }
            TokenType::StringLiteral => {
                return Ok(Some(ast::Node::String(ast::NodeString {
                    value: vec![ast::StringData::Str(token.value)],
                    column: token.position.column,
                    line: token.position.line,
                    file: token.meta,
                })));
            }
            TokenType::String => {
                let line = self.source.lines().nth(token.position.line).unwrap();
                let node = string::complex_string(token, line);
                match node {
                    Ok(node) => {
                        return Ok(Some(ast::Node::String(node)));
                    }
                    Err(error) => {
                        return Ok(Some(ast::Node::Error(error)));
                    }
                }
            }
            TokenType::Punctuation => match token.value.as_str() {
                "{" => {
                    let expr = self.parse_object_expr();
                    return Ok(Some(expr.unwrap()));
                }
                "(" => {
                    let expr = self.parse_expr();
                    if expr.is_none() {
                        return Err(token);
                    }
                    let expr = expr.unwrap();
                    let close_paren = self.expect(TokenType::Punctuation, "");
                    if close_paren.token_type == TokenType::Error || close_paren.value != ")" {
                        return Ok(Some(ast::Node::Error(ast::NodeError {
                            message: "Se esperaba un paréntesis de cierre".to_string(),
                            column: close_paren.position.column,
                            line: close_paren.position.line,
                            meta: close_paren.meta,
                        })));
                    }
                    return Ok(Some(expr));
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
                    if expr.is_none() {
                        return Err(token);
                    }
                    let expr = expr.unwrap();
                    return Ok(Some(ast::Node::UnaryFront(ast::NodeUnary {
                        operator: token.value,
                        operand: Box::new(expr),
                        column: token.position.column,
                        line: token.position.line,
                        file: token.meta,
                    })));
                }
                _ => Err(token),
            },
            _ => Err(token),
        }
    }
    fn parse_object_expr(&mut self) -> Option<ast::Node> {
        let open_brace = self.prev();
        let mut properties: Vec<ast::NodeProperty> = Vec::new();

        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "}")
        {
            let property = self.parse_object_property();
            if property.is_err() {
                return Some(ast::Node::Error(property.err().unwrap()));
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
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            }));
        }
        let close_brace = self.expect(TokenType::Punctuation, "");
        if close_brace.token_type == TokenType::Error || close_brace.value != "}" {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una llave de cierre".to_string(),
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
    fn parse_array_expr(&mut self) -> Option<ast::Node> {
        let open_bracket = self.prev();
        let mut elements: Vec<ast::NodeProperty> = Vec::new();

        while self.not_eof()
            && !(self.at().token_type == TokenType::Punctuation && self.at().value == "]")
        {
            let element = self.parse_array_property();
            if element.is_err() {
                return Some(ast::Node::Error(element.err().unwrap()));
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
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba una coma".to_string(),
                column: comma.position.column,
                line: comma.position.line,
                meta: format!("{}\0{}\0{}", comma.meta, line, comma.value),
            }));
        }
        let close_brace = self.expect(TokenType::Punctuation, "");
        if close_brace.token_type == TokenType::Error || close_brace.value != "]" {
            return Some(ast::Node::Error(ast::NodeError {
                message: "Se esperaba un corchete de cierre".to_string(),
                column: close_brace.position.column,
                line: close_brace.position.line,
                meta: close_brace.meta,
            }));
        }
        Some(ast::Node::Array(ast::NodeArray {
            elements,
            column: open_bracket.position.column,
            line: open_bracket.position.line,
            file: open_bracket.meta,
        }))
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
                if element.is_none() {
                    let line = self.source.lines().nth(token.position.line).unwrap();
                    return Err(ast::NodeError {
                        message: "Se esperaba un valor para la lista".to_string(),
                        column: token.position.column,
                        line: token.position.line,
                        meta: format!("{}\0{}\0{}", token.meta, line, token.value),
                    });
                }
                let element = element.unwrap();
                return Ok(ast::NodeProperty::Indexable(element));
            }
        }
    }
}
