use super::Parser;

use crate::ast::expr::{Expr, ExprKind::*};
use crate::ast::op::BinOp;
use crate::token::TokenKind::*;

use anyhow::{anyhow, Result};

impl Parser {
    /// expression -> equality
    pub(super) fn expression(&mut self) -> Result<Box<Expr>> {
        self.assigment()
    }

    /// assignment -> ( IDENTIFIER ( "=" | "+=" | "-=" | "*=" | "/=" | "%=" ) assignment ) | conditional
    fn assigment(&mut self) -> Result<Box<Expr>> {
        let expr = self.conditional()?;

        if self.matches(&[Equal /* TODO: more assignment tokens like += */]) {
            let lhs = expr;
            let assignment_start = lhs.region;
            let value = self.assigment()?;
            let assignment_end = value.region;

            let Variable(name) = lhs.kind else {
                return Err(anyhow!("invalid assignment target {}", value));
            };
            return Ok(Expr::new(
                Assign(name, value),
                assignment_start.to(assignment_end),
            ));
        }

        Ok(expr)
    }

    fn conditional(&mut self) -> Result<Box<Expr>> {
        if self.matches(&[If]) {
            let if_kw = self.previous();
            let condition = self.or()?;
            self.expect(OpenBrace, "expected '{' after if condition")?;
            let then_branch = self.block()?;
            let else_branch = if self.matches(&[Else]) {
                self.expect(OpenBrace, "expected '{' after else condition")?;
                Some(self.block()?)
            } else {
                None
            };

            return Ok(Expr::new(
                Cond(condition, then_branch, else_branch),
                self.region_since(if_kw),
            ));
        }

        self.or()
    }

    fn or(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.and()?;
        let expr_start = lhs.region;

        // or expressions are left-associative, so the left-hand-side is mutated as more of the or
        // symbol comes in
        while self.matches(&[DoubleBar]) {
            let rhs = self.and()?;
            let expr_end = rhs.region;
            lhs = Expr::new(Binary(BinOp::Or, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    fn and(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.equality()?;
        let expr_start = lhs.region;

        // and expressions are also left-associative, so the left-hand-side is mutated as more of the and
        // symbol comes in
        while self.matches(&[DoubleAmpersand]) {
            let rhs = self.equality()?;
            let expr_end = rhs.region;
            lhs = Expr::new(Binary(BinOp::And, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    /// equality -> comparison ( ( "!=" | "==" ) comparison )*
    fn equality(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.comparison()?;
        let expr_start = lhs.region;

        while self.matches(&[BangEqual, DoubleEqual]) {
            let op = self.previous().to_bin_op().unwrap();
            let rhs = self.comparison()?;
            let expr_end = rhs.region;
            lhs = Expr::new(Binary(op, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    /// comparison -> term ( ( '>' | ">=" | '<' | "<=" ) term )*
    fn comparison(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.term()?;
        let expr_start = lhs.region;

        while self.matches(&[Gt, Ge, Lt, Le]) {
            let op = self.previous().to_bin_op().unwrap();
            let rhs = self.term()?;
            let expr_end = rhs.region;
            lhs = Expr::new(Binary(op, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    /// term -> factor ( ( '-' | '+' ) factor )*
    fn term(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.factor()?;
        let expr_start = lhs.region;

        while self.matches(&[Minus, Plus]) {
            let op = self.previous().to_bin_op().unwrap();
            let rhs = self.factor()?;
            let expr_end = rhs.region;
            lhs = Expr::new(Binary(op, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    /// factor -> unary ( ( '/' | '*' ) unary )*
    fn factor(&mut self) -> Result<Box<Expr>> {
        let mut lhs = self.unary()?;
        let expr_start = lhs.region;

        while self.matches(&[Slash, Star, Percent]) {
            let op = self.previous().to_bin_op().unwrap();
            let rhs = self.unary()?;
            let expr_end = lhs.region;
            lhs = Expr::new(Binary(op, lhs, rhs), expr_start.to(expr_end));
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.matches(&[Bang, Minus /* TODO:, Move, Clone */]) {
            let op_token = self.previous();
            let expr_start = op_token.region;
            let op = op_token.to_unary_op().unwrap();
            let rhs = self.unary()?;
            let expr_end = rhs.region;
            Ok(Expr::new(Unary(op, rhs), expr_start.to(expr_end)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Box<Expr>> {
        let (fn_name_token, fn_name) = match self.peek().kind {
            Ident(name) if self.look_ahead(1) == OpenParen => (self.eat(), name),
            _ => return self.primary(),
        };

        // eat open paren '('
        self.eat();

        let mut args = Vec::new();

        if !self.check(CloseParen) {
            args.push(self.expression()?);
            while self.matches(&[Comma]) {
                args.push(self.expression()?);
            }
        }
        let close_paren = self.expect(CloseParen, "expected ')' after argument list")?;

        Ok(Expr::new(
            Call(fn_name, args),
            self.region_from(fn_name_token, close_paren),
        ))
    }

    fn primary(&mut self) -> Result<Box<Expr>> {
        match self.peek().kind {
            Ident(name) => {
                self.eat();
                Ok(Expr::new(Variable(name), self.previous()))
            }
            Literal(_) => {
                self.eat();
                Ok(Expr::from_lit(self.previous()).unwrap())
            }

            // NOTE: i might consider creating an ast object for parenthesised expressions
            OpenParen => {
                self.eat();
                let expr = self.expression();
                self.expect(CloseParen, "expected ')' to close expression")?;
                expr
            }

            _ => Err(anyhow!("expected an expression {}", self.peek())),
        }
    }
}
