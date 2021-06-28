use crate::{ast::Stmt, lexer::Token};

use super::{ParseError, ParseErrorKind, ParseResult, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_stmt(&mut self) -> ParseResult<Box<Stmt>> {
        let stmt = match self.peek() {
            Token::Var => {
                self.eat();
                self.parse_var_def()?
            }
            Token::Ret => {
                self.eat();
                let expr = self.parse_expr()?;
                Box::new(Stmt::Ret(expr))
            }
            _ => {
                let expr = *self.parse_expr()?;
                Box::new(Stmt::Expr(expr))
            }
        };

        if self.eat() != Token::Semicolon {
            return Err(self.syntax_error("expected `;` to terminate statement"));
        }

        Ok(stmt)
    }

    fn invalid_module_error(&mut self, path: String) -> ParseError {
        use ParseErrorKind::InvalidModPath;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: InvalidModPath(path),
        };
        self.errors.push(err.clone());
        err
    }
    
    pub(super) fn parse_impt_stmt(&mut self) -> ParseResult<String> {
        if let Token::Str(path) = self.eat() {
            if let Some(name) = path.split('.').last() {
                if name.is_empty() {
                    Err(self.invalid_module_error(path))
                } else if self.sym_tbl.insert(name.to_owned()) {
                    Ok(path)
                } else {
                    return Err(self.redefined_symbol_error(name));
                }
            } else {
                Err(self.invalid_module_error(path))
            }
        } else {
            Err(self.syntax_error("expected a module path after after an `@impt` statement"))
        }
    }
}
