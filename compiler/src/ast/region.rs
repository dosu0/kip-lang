use super::expr::Expr;
use super::stmt::{FuncProto, Stmt};

use crate::source::Source;
use crate::token::Token;

use std::cmp;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

/// A region of a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    start: usize,
    len: usize,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start(), self.end())
    }
}

impl Region {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            len: end - start,
        }
    }

    /// Get the region's starting offset.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Calculate the region's ending point.
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    /// Get the region's length.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn to<E: Into<Region>>(self, end: E) -> Region {
        let end = end.into();
        Region::new(
            cmp::min(self.start, end.start),
            cmp::max(self.end(), end.end()),
        )
    }

    /// Calculate the region's starting line and column
    /// NOTE: This is quite an expensive calculation
    pub fn line_column(&self, source: &Source) -> LineColumn {
        let mut pos = LineColumn { line: 1, column: 0 };

        for (n, c) in source.contents.bytes().enumerate() {
            pos.column += 1;
            if c == b'\n' {
                pos.line += 1;
                pos.column = 0;
            }

            if n == self.start {
                break;
            }
        }

        pos
    }

    pub fn to_str<'a>(&self, source: &'a Source) -> &'a str {
        &source.contents[self.start..self.end()]
    }

    pub fn to_range(&self) -> Range<usize> {
        self.start..self.end()
    }
}

impl From<Token> for Region {
    fn from(token: Token) -> Self {
        token.region
    }
}

impl From<&Expr> for Region {
    fn from(expr: &Expr) -> Self {
        expr.region
    }
}

impl From<&Stmt> for Region {
    fn from(stmt: &Stmt) -> Self {
        stmt.region
    }
}

impl From<FuncProto> for Region {
    fn from(proto: FuncProto) -> Self {
        proto.region
    }
}
