use crate::lexer::Token;
use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BINOP_PRECEDENCE: HashMap<char, i32> = {
        let mut m = HashMap::new();
        m.insert('=', 2);
        m.insert('<', 10);
        m.insert('>', 10);
        m.insert('+', 20);
        m.insert('-', 20);
        m.insert('/', 40);
        m.insert('*', 40);
        m
    };
}

pub fn get_tok_prec(token: Token) -> i32 {
    match token {
        Token::Op(c) => *BINOP_PRECEDENCE.get(&c).unwrap_or(&-1),
        _ => -1,
    }
}
