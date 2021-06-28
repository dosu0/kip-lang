use super::{ParseResult, Parser};
use crate::ast::{Expr, LitKind};
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
    fn parse_ident_expr(&mut self, ident_name: &str) -> ParseResult<Box<Expr>> {
        if self.peek() == Token::OpenParen {
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

            Ok(Box::new(Expr::Call(ident_name.to_owned(), args)))
        } else {
            Ok(Box::new(Expr::Var(ident_name.to_owned())))
        }
    }

    pub(super) fn parse_expr(&mut self) -> ParseResult<Box<Expr>> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    fn parse_primary(&mut self) -> ParseResult<Box<Expr>> {
        use Expr::Lit;
        use LitKind::*;

        match self.eat() {
            Token::Ident(name) => self.parse_ident_expr(&name),
            Token::Number(val) => Ok(Box::new(Lit(Int(val)))),
            Token::Str(val) => Ok(Box::new(Lit(Str(val)))),
            Token::OpenParen => self.parse_paren_expr(),
            tok => {
                Err(self.syntax_error(&format!("expected an expression, instead found `{}`", tok)))
            }
        }
    }

    fn parse_bin_op_rhs(&mut self, expr_prec: u32, mut lhs: Box<Expr>) -> ParseResult<Box<Expr>> {
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

            lhs = Box::new(Expr::Binary(bin_op, lhs, rhs));
        }
    }
}
