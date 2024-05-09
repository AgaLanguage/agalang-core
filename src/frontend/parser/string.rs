use super::{ast::{NodeError, NodeString, StringData}, Parser};
use crate::frontend::lexer::TokenType;
use util::Token;

fn is_alpha(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$' || c.is_numeric()
}
pub fn complex_string(token_string: Token<TokenType>, line: &str) -> Result<NodeString, NodeError> {
    let string = token_string.value;
    let mut result = Vec::new();
    let mut current = String::new();
    let mut is_id = false;
    let mut i = 0;
    while i < string.len() {
        let c = string.chars().nth(i).unwrap();
        i += 1;
        if c == '}' && is_id == false {
            let nc = string.chars().nth(i);
            i += 1;
            if nc == None {
                return Err(NodeError {
                    message: "No se encontro la apertura de el identificador".to_string(),
                    column: token_string.position.column + i,
                    line: token_string.position.line,
                    meta: format!(
                        "{}\0{}\0{}",
                        token_string.meta, line, string
                    ),
                });
            }
            let nc = nc.unwrap();
            if nc == '}' {
              current.push('}');
                continue;
            }
        }
        if c != '{' && is_id == false {
            current.push(c);
            continue;
        }
        if is_id {
            let node = Parser::new(current.clone(), &token_string.meta).produce_ast();
            if node.is_error() {
                let node = node.get_error().unwrap();
                return Err(NodeError {
                    message: node.message,
                    column: token_string.position.column + i,
                    line: token_string.position.line,
                    meta: format!(
                        "{}\0{}\0{}",
                        token_string.meta, line, string
                    ),
                });
            }
            if c == '}' {
                result.push(StringData::Id(node.to_box()));
                current.clear();
                is_id = false;
                continue;
            }
            if is_alpha(c) {
                current.push(c);
                continue;
            }
        }
        let nc = string.chars().nth(i);
        i += 1;
        if nc == None {
            return Err(NodeError {
                message: "Se esperaba un caracter literal".to_string(),
                column: token_string.position.column + i,
                line: token_string.position.line,
                meta: format!(
                    "{}\0{}\0{}",
                    token_string.meta, line, string
                ),
            });
        }
        let nc = nc.unwrap();
        if nc == '{' {
            current.push('{');
            continue;
        }
        if nc == '}' {}
        is_id = true;
        result.push(StringData::Str(current.clone()));
        current.clear();
        current.push(nc);
    }
    if is_id {
        return Err(NodeError {
            message: "Se esperaba cierre del identificador".to_string(),
            column: token_string.position.column + i,
            line: token_string.position.line,
            meta: format!(
                "{}\0{}\0{}",
                token_string.meta, line, string
            ),
        });
    }
    if current.len() > 0 {
        result.push(StringData::Str(current));
    }
    Ok(NodeString {
        value: result,
        column: token_string.position.column,
        line: token_string.position.line,
        file: token_string.meta,
    })
}
