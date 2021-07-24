use crate::ast::Block;
use crate::lexer::Token;

use super::{ParseResult, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self) -> ParseResult<Box<Block>> {
        if self.eat() != Token::OpenBrace {
            return Err(self.syntax_error("expected a block"));
        }

        let mut stmts = Vec::new();

        while self.peek() != Token::CloseBrace {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        // eat the closing brace
        self.eat();

        Ok(Box::new(Block { stmts }))
    }
}
