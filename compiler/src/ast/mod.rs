//! Abstract Syntax Tree (AST) nodes

use crate::name::Name;

pub mod expr;
pub mod lit;
pub mod op;
pub mod region;
pub mod stmt;
pub mod ty;
pub mod visit;

pub use expr::{Expr, ExprKind};
pub use lit::Lit;
pub use op::{BinOp, UnOp};
pub use region::Region;
pub use stmt::{Block, Stmt, StmtKind};
pub use ty::Type;
pub use visit::{ExprVisitor, StmtVisitor};

#[derive(Clone, Copy)]
pub struct Ident {
    pub name: Name,
    pub region: Region,
}

impl Ident {
    pub fn new(name: Name, region: Region) -> Self {
        Self { name, region }
    }
}
