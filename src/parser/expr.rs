use super::{ParseResult, Parser};
use crate::ast::{Expr, ExprKind, LitKind, Region};
use crate::lexer::Token;

impl<'a> Parser<'a> {
    /// paren_expr -> `(` `expr` `)`
    fn parse_paren_expr(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.parse_expr()?;

        if self.eat() != Token::CloseParen {
            return Err(self.syntax_error("expected a `)`"));
        }

        Ok(expr)
    }

    /// ident_expr -> var | call | assign
    fn parse_ident_expr(&mut self, ident_name: String) -> ParseResult<Box<Expr>> {
        use ExprKind::{Assign, Call, Var};
        let start = self.tokens.offset();
        let kind = if self.peek() == Token::OpenParen {
            self.eat();
            // function call
            let mut args = Vec::new();

            loop {
                if self.peek() != Token::CloseParen {
                    args.push(self.parse_expr()?);
                } else {
                    self.eat();
                    break;
                }

                match self.eat() {
                    // End of argument list
                    Token::CloseParen => break,
                    // More arguments ...
                    Token::Comma => continue,
                    _ => return Err(self.syntax_error("expected a `,` or `)` in an argument list")),
                }
            }

            Call(ident_name, args)
        } else if self.peek() == /* TODO: more assign operators */ Token::Equal {
            self.eat();
            let init = self.parse_expr()?;
            Assign(ident_name, init)
        } else {
            Var(ident_name)
        };

        Ok(Box::new(Expr {
            kind,
            region: Region {
                start,
                end: self.tokens.offset(),
            },
        }))
    }

    fn parse_cond_expr(&mut self) -> ParseResult<Box<Expr>> {
        use ExprKind::Cond;
        let mut region = Region {
            start: self.tokens.offset(),
            end: 0,
        };

        let condition = self.parse_expr()?;
        let if_block = self.parse_block()?;
        let else_block = if let Token::Else = self.eat() {
            Some(self.parse_block()?)
        } else {
            None
        };

        region.end = self.tokens.offset();

        let kind = Cond(condition, if_block, else_block);

        Ok(Box::new(Expr { region, kind }))
    }

    pub(super) fn parse_expr(&mut self) -> ParseResult<Box<Expr>> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    fn parse_primary(&mut self) -> ParseResult<Box<Expr>> {
        use ExprKind::Lit;
        use LitKind::*;

        let start = self.tokens.offset();
        match self.eat() {
            Token::Ident(name) => {
                let mut expr = self.parse_ident_expr(name)?;
                expr.region.start = start;
                Ok(expr)
            }
            Token::Number(val) => {
                let kind = Lit(Int(val));
                let region = Region {
                    start,
                    end: self.tokens.offset(),
                };
                Ok(Box::new(Expr { kind, region }))
            }
            Token::Str(val) => {
                let kind = Lit(Str(val));
                let region = Region {
                    start,
                    end: self.tokens.offset(),
                };
                Ok(Box::new(Expr { kind, region }))
            }
            Token::Char(ch) => {
                let kind = Lit(Char(ch));
                let region = Region {
                    start,
                    end: self.tokens.offset(),
                };
                Ok(Box::new(Expr { kind, region }))
            }
            Token::OpenParen => self.parse_paren_expr(),
            Token::If => {
                let mut expr = self.parse_cond_expr()?;
                expr.region.start = start;
                Ok(expr)
            }
            tok => {
                Err(self.syntax_error(&format!("expected an expression, instead found `{}`", tok)))
            }
        }
    }

    fn parse_bin_op_rhs(&mut self, expr_prec: u32, mut lhs: Box<Expr>) -> ParseResult<Box<Expr>> {
        use ExprKind::Binary;

        loop {
            let bin_op = match self.peek().to_bin_op() {
                Some(o) => o,
                _ => return Ok(lhs),
            };

            self.eat();

            let tok_prec = bin_op.get_prec();

            if tok_prec < expr_prec {
                return Ok(lhs);
            };

            let mut rhs = self.parse_primary()?;

            match self.peek().to_bin_op() {
                Some(o) => {
                    let next_prec = o.get_prec();
                    if tok_prec < next_prec {
                        rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs)?;
                    }
                }
                None => rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs)?,
            }

            let start = lhs.region.start;
            lhs = Box::new(Expr {
                kind: Binary(bin_op, lhs, rhs),
                region: Region {
                    start,
                    end: self.tokens.offset(),
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::BinOp;
    use crate::ast::ExprKind::*;
    use crate::ast::StmtKind;
    use crate::lexer::TokenStream;
    use crate::source::Source;

    use super::*;

    #[test]
    fn assign_expression() {
        let input = "a = b = c";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();
        if let Assign(name, init) = expr.kind {
            assert_eq!(name, "a");
            if let Assign(name, init) = init.kind {
                assert_eq!(name, "b");
                if let Var(name) = init.kind {
                    assert_eq!(name, "c");
                } else {
                    panic!("expected an variable");
                }
            } else {
                panic!("expected an assignment");
            }
        } else {
            panic!("expected an assignment");
        }
    }
    #[test]
    fn call() {
        let input = "foo(bar, baz)";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();

        if let Call(name, args) = expr.kind {
            assert_eq!(name, "foo");
            if let Var(name) = &args[0].kind {
                assert_eq!(name, "bar");
            }
            if let Var(name) = &args[1].kind {
                assert_eq!(name, "baz");
            }
        }
    }

    #[test]
    fn paren_expressions() {
        let input = "(a + b)";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();

        if let Binary(op, lhs, rhs) = expr.kind {
            assert_eq!(op, BinOp::Add);

            if let Var(name) = lhs.kind {
                assert_eq!(name, "a");
            } else {
                panic!("expected variable reference");
            }

            if let Var(name) = rhs.kind {
                assert_eq!(name, "b");
            } else {
                panic!("expected variable reference");
            }
        } else {
            panic!("expected binary expression");
        }
    }

    #[test]
    fn conditionals() {
        use LitKind::*;
        let input = "\
        if x >= 0 {\n\
            positive();\n\
        } else {\n\
            negative();\n\
        }";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();

        if let Cond(condition, if_block, else_block) = expr.kind {
            if let Binary(op, lhs, rhs) = condition.kind {
                assert_eq!(op, BinOp::Ge);
                if let Var(name) = lhs.kind {
                    assert_eq!(name, "x");
                } else {
                    panic!("expected variable reference");
                }

                if let Lit(Int(num)) = rhs.kind {
                    assert_eq!(num, 0);
                } else {
                    panic!("expected an integer literal");
                }
            } else {
                panic!("expected binary expression");
            }

            if let StmtKind::Expr(expr) = &if_block.stmts.last().unwrap().kind {
                if let Call(name, args) = &expr.kind {
                    assert_eq!(name, "positive");
                    assert!(args.is_empty());
                }
            }

            if let StmtKind::Expr(expr) = &else_block.unwrap().stmts.last().unwrap().kind {
                if let Call(name, args) = &expr.kind {
                    assert_eq!(name, "negative");
                    assert!(args.is_empty());
                }
            }
        } else {
            panic!("expected a conditional expression");
        }
    }
}
