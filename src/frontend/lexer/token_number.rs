use super::TokenType;

fn is_constant(c: char) -> bool {
    c == 'i' || c == 'e' || c == 'π'
}
fn is_number(c: char, use_dot: bool) -> bool {
    c.is_numeric() || (!use_dot && c == '.') || is_constant(c)
}

fn number_literal(
    position: util::Position,
    line: String,
    meta: String,
) -> (util::Token<TokenType>, usize) {
    let col = position.column;
    let mut i = col;
    let mut use_dot = false;
    let mut value = String::new();
    while i < line.len() {
        let c = line.chars().nth(i);
        if c.is_none() {
            break;
        }
        let c = c.unwrap();
        if !is_number(c, use_dot) {
            break;
        }
        if c == '.' {
            use_dot = true;
        }
        value.push(c);
        i += 1;
    }
    let token = util::Token {
        token_type: TokenType::NumberLiteral,
        position,
        value,
        meta,
    };
    (token, i - col - 1)
}

fn number_base(
    c: char,
    pos: util::Position,
    line: String,
    meta: String,
) -> (util::Token<TokenType>, usize) {
    let col = pos.column;
    let mut i = col + 1;
    let mut base = 10;
    if c == '0' && i < line.len() {
        let c = line.chars().nth(i);
        if c.is_none() {
            return (
                util::Token {
                    token_type: TokenType::NumberLiteral,
                    position: pos,
                    value: "0".to_string(),
                    meta,
                },
                i - col - 1,
            );
        }
        let c = c.unwrap();
        if c == 'b' {
            base = 2;
            i += 1;
            if let Some('y') = line.chars().nth(i) {
                i += 1;
                let mut value = String::new();
                let mut x = 1;
                while x <= 8 {
                    let bit = line.chars().nth(i);
                    i += 1;
                    if bit.is_none() {
                        break;
                    }
                    let bit = bit.unwrap();
                    if bit == '0' || bit == '1' {
                        value.push(bit);
                        x += 1;
                    } else if bit != '_' {
                        break;
                    }
                }
                return if x == 1 {
                    (
                        util::Token {
                            token_type: TokenType::Error,
                            position: pos,
                            value: format!("No se pudo analizar el byte"),
                            meta,
                        },
                        i - col - 1,
                    )
                } else {
                    (
                        util::Token {
                            token_type: TokenType::Byte,
                            position: pos,
                            value,
                            meta,
                        },
                        i - col - 1,
                    )
                };
            }
        } else if c == 'o' {
            base = 8;
            i += 1;
        } else if c == 'd' {
            base = 10;
            i += 1;
        } else if c == 'x' {
            base = 16;
            i += 1;
        } else if c == '$' {
            i += 1;
            if i >= line.len() {
                // not i < line.len()
                return (
                    util::Token {
                        token_type: TokenType::Error,
                        position: pos,
                        value: "Se esperaba un número base".to_string(),
                        meta: format!("{meta}\0{line}\00$"),
                    },
                    0,
                );
            }
            let mut base_str = String::new();
            while i < line.len() {
                let c = line.chars().nth(i);
                if c.is_none() {
                    break;
                }
                let c = c.unwrap();
                if c.is_digit(10) {
                    break;
                }
                base_str.push(c);
                i += 1;
            }
            if base_str.len() == 0 {
                return (
                    util::Token {
                        token_type: TokenType::Error,
                        position: pos,
                        value: "Se esperaba un número base".to_string(),
                        meta: format!("{meta}\0{line}\00$"),
                    },
                    i - col - 1,
                );
            }
            let base_number = base_str.parse::<u8>();
            if base_number.is_err() {
                return (
                    util::Token {
                        token_type: TokenType::Error,
                        position: pos,
                        value: "Se esperaba un número en base 10".to_string(),
                        meta: format!("{meta}\0{line}\00${base_str}"),
                    },
                    i - col - 1,
                );
            }
            let base_number = base_number.unwrap();
            if base_number < 2 || base_number > 36 {
                return (
                    util::Token {
                        token_type: TokenType::Error,
                        position: pos,
                        value: "La base debe estar entre 2 y 36".to_string(),
                        meta: format!("{meta}\0{line}\00${base_str}"),
                    },
                    i - col - 1,
                );
            }
            base = base_number;
            let value_char = line.chars().nth(i);
            if value_char == None {
                return (
                    util::Token {
                        token_type: TokenType::Error,
                        position: pos,
                        value: "Se esperaba un \"~\" para el valor".to_string(),
                        meta: format!("{meta}\0{line}\00${base}"),
                    },
                    i - col - 1,
                );
            }
            if value_char.unwrap() == '~' {
                i += 1;
            }
        }
    }

    // save the first index of the value
    let value_index = i;

    while i < line.len() {
        let c = line.chars().nth(i);
        if c.is_none() {
            break;
        }
        let c = c.unwrap();
        if c.is_digit(base as u32) {
            break;
        }
        i += 1;
    }
    let value = line[value_index..i].to_string();
    let token = util::Token {
        token_type: TokenType::Number,
        position: pos,
        value: format!("0${}~{}", base, value),
        meta,
    };
    (token, i - col - 1)
}

pub fn token_number(
    c: char,
    pos: util::Position,
    line: String,
    file_name: String,
) -> (util::Token<TokenType>, usize) {
    if c == '0' {
        let next = line.chars().nth(pos.column + 1);
        if next != None && util::is_valid_char("bodx$", next.unwrap()) {
            return number_base(c, pos, line, file_name);
        }
    }
    number_literal(pos, line, file_name)
}
