use crate::{
    ast::{Region, Stmt, StmtKind, Type},
    lexer::Token,
    symbol::Symbol,
};

use super::{ParseError, ParseErrorKind, ParseResult, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_stmt(&mut self) -> ParseResult<Box<Stmt>> {
        let mut region = Region {
            start: self.tokens.offset(),
            end: 0,
        };
        let mut should_end_with_semicolon = true;
        let stmt = match self.peek() {
            Token::Var => {
                region.start = self.tokens.offset();
                self.eat();
                let kind = self.parse_var_def()?.kind;
                region.end = self.tokens.offset();
                Box::new(Stmt { kind, region })
            }
            Token::Ret => {
                region.start = self.tokens.offset();
                self.eat();
                let expr = self.parse_expr()?;
                let kind = StmtKind::Ret(expr);
                region.end = self.tokens.offset();
                Box::new(Stmt { kind, region })
            }
            _ => {
                let expr = *self.parse_expr()?;
                should_end_with_semicolon = !expr.kind.is_cond();
                let kind = StmtKind::Expr(expr);
                region.end = self.tokens.offset();
                Box::new(Stmt { kind, region })
            }
        };

        if should_end_with_semicolon && self.eat() != Token::Semicolon {
            return Err(self.syntax_error("expected `;` to terminate statement"));
        }

        Ok(stmt)
    }

    fn parse_var_def(&mut self) -> ParseResult<Box<Stmt>> {
        let mut region = Region {
            start: self.tokens.offset(),
            end: 0,
        };
        let name = if let Token::Ident(name) = self.eat() {
            name
        } else {
            return Err(self.syntax_error("expected a variable name in variable declaration"));
        };

        if self.eat() != Token::Colon {
            return Err(self.syntax_error("expected type annotation in variable definition"));
        }

        let ty = self.parse_type_annotation()?;

        if self.eat() != Token::Equal {
            return Err(self.syntax_error("expected `=` in variable declaration"));
        }

        let init = self.parse_expr()?;

        // symbol has already been defined
        if let Some(sym) = self.sym_tbl.get(&name) {
            let e = match sym {
                Symbol::Func(_, _) => {
                    self.redefined_symbol_error(&name, Some("a function with the same name exists"))
                }
                Symbol::Var(_) => self.redefined_symbol_error(
                    &name,
                    Some("another variable with the same name exists"),
                ),
            };
            Err(e)
        } else {
            region.end = self.tokens.offset();
            self.sym_tbl.insert(name.clone(), Symbol::Var(ty));
            let kind = StmtKind::VarDef(name, init);
            Ok(Box::new(Stmt { kind, region }))
        }
    }

    fn invalid_module_error(&mut self, path: String) -> ParseError {
        use ParseErrorKind::InvalidModPath;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: InvalidModPath(path),
            hint: None,
        };
        self.errors.push(err.clone());
        err
    }

    pub(super) fn parse_impt_stmt(&mut self) -> ParseResult<String> {
        if let Token::Str(path) = self.eat() {
            if let Some(name) = path.split('.').last() {
                if name.is_empty() {
                    Err(self.invalid_module_error(path))
                } else if self.sym_tbl.contains_key(name) {
                    Err(self.redefined_symbol_error(name, Some("remove this import statement")))
                } else {
                    self.sym_tbl
                        .insert(name.to_owned(), Symbol::Var(Type::Other(name.to_owned())));
                    Ok(path)
                }
            } else {
                Err(self.invalid_module_error(path))
            }
        } else {
            Err(self.syntax_error("expected a module path after after an `@impt` statement"))
        }
    }
}
