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

    /// ident_expr -> ident | call_expr
    fn parse_ident_expr(&mut self, ident_name: String) -> ParseResult<Box<Expr>> {
        let start = self.tokens.offset();
        let kind = if self.peek() == Token::OpenParen {
            self.eat();
            // function call
            let mut args = Vec::new();

            loop {
                args.push(self.parse_expr()?);

                match self.eat() {
                    // End of argument list
                    Token::CloseParen => break,
                    // More arguments ...
                    Token::Comma => continue,
                    _ => return Err(self.syntax_error("expected a `,` or `)` in an argument list")),
                }
            }

            ExprKind::Call(ident_name, args)
        } else {
            ExprKind::Var(ident_name)
        };

        Ok(Box::new(Expr {
            kind,
            region: Region {
                start,
                end: self.tokens.offset(),
            },
        }))
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
            Token::OpenParen => self.parse_paren_expr(),
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
