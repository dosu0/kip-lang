//! What:
//! This is a top-down recursive-descent parser
//!
//! How:
//! This parser walks down the grammar tree starting at the "module" level then ending at primary expressions.
//!
//! Why:
//! I choose this method so that my grammar rules could directly parallel to the parsing code. In
//! my opinion this leads for a simple, yet robust way of implementing kip's parser.

pub mod expr;
pub mod stmt;
#[cfg(test)]
mod tests;
pub mod ty;

use anyhow::{bail, Result};

use crate::ast::{Region, Stmt};
use crate::generate_error_message;
use crate::name::Name;
use crate::source::Source;
use crate::token::{Token, TokenKind, TokenKind::*};

pub struct Parser {
    source: Source,
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: &Source) -> Self {
        Self {
            // TODO: remove clone
            source: source.clone(),
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Result<Box<Stmt>>> {
        let mut decls = Vec::new();

        while !self.is_eof() {
            decls.push(self.declaration());
        }

        decls
    }

    // Synchronizes the parser so that it can continue parsing
    fn sync(&mut self) {
        self.eat();

        while !self.is_eof() {
            if self.previous().kind == Semicolon {
                return;
            }
            match self.peek().kind {
                Extern | Func | Var | If | Ret => return,
                _ => {
                    self.eat();
                }
            };
        }
    }

    // if the next token is any of token kinds, the token is eaten and true is returned
    // otherwise returns false
    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        if kinds.contains(&self.peek().kind) {
            self.eat();
            true
        } else {
            false
        }
    }

    /// Get the current un-eaten token
    fn peek(&self) -> Token {
        self.tokens[self.current]
    }

    /// Look ahead `n` tokens
    fn look_ahead(&self, n: usize) -> Token {
        self.tokens[self.current + n]
    }

    /// Return the current token and advance to the next token
    fn eat(&mut self) -> Token {
        if self.is_eof() {
            self.peek()
        } else {
            self.current += 1;
            self.previous()
        }
    }

    /// Get the previous token
    #[inline]
    fn previous(&self) -> Token {
        self.tokens[self.current - 1]
    }

    /// Peeks at the next token and returns true if it matches the given kind
    /// Returns false on Eof or if the token kinds don't match up
    fn check(&self, token: TokenKind) -> bool {
        self.peek() == token
    }

    fn is_eof(&self) -> bool {
        self.peek() == Eof
    }

    fn generate_error_message(&self, base_message: &'static str) -> String {
        generate_error_message(base_message, &self.source, self.peek().region)
    }

    ///
    fn expect(&mut self, kind: TokenKind, message: &'static str) -> Result<Token> {
        if self.check(kind) {
            Ok(self.eat())
        } else {
            bail!("{}", self.generate_error_message(message))
        }
    }

    fn expect_ident(&mut self, message: &'static str) -> Result<Name> {
        match self.peek().kind {
            Ident(name) => {
                self.eat();
                Ok(name)
            }
            _ => bail!("{}", self.generate_error_message(message)),
        }
    }

    fn region_since(&self, start: Token) -> Region {
        start.region.to(self.previous().region)
    }

    fn region_from(&self, start: Token, end: Token) -> Region {
        start.region.to(end.region)
    }

    // Get a reference to the parser's symbol table.
    // #[cfg(test)]
    // pub fn sym_tbl(&self) -> &SymbolTable {
    //  &self.sym_tbl
    // }
}
