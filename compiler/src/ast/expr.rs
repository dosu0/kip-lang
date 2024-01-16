use super::lit::Lit;
use super::op::{BinOp, UnOp};
use super::region::Region;
use super::stmt::Block;

use crate::name::Name;
use crate::token::{Token, TokenKind};

use std::fmt;

/// expression (expr)
/// expr -> lit | var | binary | call | conditional | assign
#[derive(Debug, PartialEq)]
pub enum ExprKind {
    Lit(Lit),
    Variable(Name),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    #[allow(clippy::vec_box)]
    Call(Name, Vec<Box<Expr>>),
    Cond(Box<Expr>, Block, Option<Block>),
    Assign(Name, Box<Expr>),
}

impl From<i64> for ExprKind {
    fn from(value: i64) -> Self {
        Self::Lit(Lit::Int(value))
    }
}

impl fmt::Display for ExprKind {
    // TODO: don't use debug formatting on vector types
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Lit(lit) => write!(f, "Lit({})", lit),
            ExprKind::Variable(name) => write!(f, "Variable(\"{}\")", name),
            ExprKind::Unary(op, rhs) => write!(f, "Unary({}, {})", op, rhs),
            ExprKind::Binary(op, lhs, rhs) => write!(f, "Binary({}, {}, {})", op, lhs, rhs),
            ExprKind::Call(fn_name, args) => write!(f, "Call({}, {:?})", fn_name, args),
            ExprKind::Cond(condition, then_branch, None) => {
                write!(f, "Cond({}, {:?})", condition, then_branch)
            }
            ExprKind::Cond(condition, then_branch, Some(else_branch)) => write!(
                f,
                "Cond({}, {:?}, {:?})",
                condition, then_branch, else_branch
            ),
            ExprKind::Assign(var_name, rhs) => write!(f, "Assign({}, {})", var_name, rhs),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub region: Region,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl Expr {
    pub fn new<R: Into<Region>>(kind: ExprKind, region: R) -> Box<Self> {
        Box::new(Expr {
            kind,
            region: region.into(),
        })
    }

    pub fn from_lit(token: Token) -> Option<Box<Self>> {
        match token.kind {
            TokenKind::Literal(lit) => Some(Expr::new(ExprKind::Lit(lit), token.region)),
            _ => None,
        }
    }

    /// Returns `true` if the expr_kind is [`Cond`].
    pub fn is_cond(&self) -> bool {
        matches!(self.kind, ExprKind::Cond(..))
    }

    #[inline]
    pub fn to<E: Into<Region>>(&self, end: E) -> Region {
        self.region.to(end)
    }

    /* #[inline]
    pub fn to(&self, end: &Expr) -> Region {
        self.region.to(end.region)
    } */
}
