pub use self::TokenKind::*;
use crate::ast::{BinOp, Expr, Lit, Region, UnOp};
use crate::name::Name;

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub region: Region,
}

impl PartialEq<TokenKind> for Token {
    fn eq(&self, other: &TokenKind) -> bool {
        self.kind == *other
    }
}

impl Token {
    pub fn new(kind: TokenKind, region: Region) -> Self {
        Self { kind, region }
    }

    /// convert lexical token to its corresponding binary operator
    pub fn to_bin_op(&self) -> Option<BinOp> {
        match self.kind {
            Plus => Some(BinOp::Add),
            Minus => Some(BinOp::Sub),
            Slash => Some(BinOp::Div),
            Star => Some(BinOp::Mul),
            BangEqual => Some(BinOp::Ne),
            DoubleEqual => Some(BinOp::Eq),
            Percent => Some(BinOp::Mod),
            Gt => Some(BinOp::Gt),
            Ge => Some(BinOp::Ge),
            Lt => Some(BinOp::Lt),
            Le => Some(BinOp::Le),
            DoubleAmpersand => Some(BinOp::And),
            DoubleBar => Some(BinOp::Or),
            _ => None,
        }
    }

    /// convert lexical token to its corresponding binary operator
    pub fn to_unary_op(&self) -> Option<UnOp> {
        match self.kind {
            Bang => Some(UnOp::Not),
            Minus => Some(UnOp::Neg),
            _ => None,
        }
    }

    /* pub fn ident(&self) -> Option<Ident> {
        match self.kind {
            Ident(name) => Some(Ident::new(name)),
            _ => None,
        }
    } */

    pub fn to(&self, end: &Expr) -> Region {
        self.region.to(end.region)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Func => "func".fmt(f),
            Extern => "extern".fmt(f),
            Var => "var".fmt(f),
            Ret => "return".fmt(f),
            If => "if".fmt(f),
            Else => "else".fmt(f),
            While => "while".fmt(f),
            Impt => "@impt".fmt(f),
            Expt => "@expt".fmt(f),
            Ident(ident) => ident.fmt(f),
            Literal(lit) => lit.fmt(f),
            OpenParen => ')'.fmt(f),
            CloseParen => '('.fmt(f),
            OpenBrace => '{'.fmt(f),
            CloseBrace => '}'.fmt(f),
            Colon => ':'.fmt(f),
            Comma => ','.fmt(f),
            Semicolon => ';'.fmt(f),
            Plus => '+'.fmt(f),
            Minus => '-'.fmt(f),
            Star => '*'.fmt(f),
            Slash => '/'.fmt(f),
            Dot => '.'.fmt(f),
            Percent => "%".fmt(f),
            Gt => '>'.fmt(f),
            Ge => ">=".fmt(f),
            Lt => "<".fmt(f),
            Le => "<=".fmt(f),
            Equal => '='.fmt(f),
            Ampersand => '&'.fmt(f),
            DoubleAmpersand => "&&".fmt(f),
            Bar => '|'.fmt(f),
            DoubleBar => "||".fmt(f),
            Eof => "end of file".fmt(f),
            DoubleEqual => "==".fmt(f),
            Bang => '!'.fmt(f),
            BangEqual => "!=".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenKind {
    // key / reserved words
    Func,
    Extern,
    Var,
    If,
    Else,
    While,
    Ret,
    Impt,
    Expt,

    Ident(Name),
    Literal(Lit),
    // symbols
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,

    // operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `=`
    Equal,
    /// `==`
    DoubleEqual,
    /// `%`
    Percent,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// '.'
    Dot,
    /// `&`
    Ampersand,
    /// `&&`
    DoubleAmpersand,
    /// `|`
    Bar,
    /// `||`
    DoubleBar,
    /// `!`
    Bang,
    /// `!=`
    BangEqual,
    Eof,
    // TODO: add an error token type
    // Err(LexError)
}
