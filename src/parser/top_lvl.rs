use crate::ast::FuncDef;
use crate::lexer::Token;
use crate::parser::Parser;

impl<'a> Parser<'a> {
    fn handle_func(&mut self) {
        match self.parse_func() {
            Ok(func) => {
                eprintln!("parsed a function definition: {:#?}", func);
            }
            Err(e) => {
                eprintln!("[kip::parser] error: {} ({:?})", e, self.errors);
                self.eat();
            }
        }
    }

    fn handle_extern(&mut self) {
        match self.parse_extern() {
            Ok(proto) => eprintln!("parsed an extern statement: {:#?}", proto),
            Err(e) => {
                eprintln!("[kip::parser] error: {}", e);
                self.eat();
            }
        }
    }

    fn handle_top_lvl_expr(&mut self) {
        match self.parse_top_lvl_expr().as_deref() {
            Ok(FuncDef { body: expr, .. }) => {
                eprintln!("parsed a top level expression: {:#?}", expr)
            }
            Err(e) => {
                eprintln!("[kip::parser] error: {}", e);
                self.eat();
            }
        }
    }

    fn handle_impt_stmt(&mut self) {
        match self.parse_impt_stmt() {
            Ok(path) => eprintln!("parsed an `@impt` statement: imported {}", path),
            Err(e) => eprintln!("[kip::parser] error: {}", e),
        }
    }
    pub fn parse(&mut self) {
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::Impt => {
                    self.eat();
                    self.handle_impt_stmt();
                }
                Token::Func => {
                    self.eat();
                    self.handle_func();
                }
                Token::Extern => {
                    self.eat();
                    self.handle_extern();
                }
                _ => self.handle_top_lvl_expr(),
            }
        }
    }
}
