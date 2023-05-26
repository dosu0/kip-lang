use super::expr::Expr;
use super::region::Region;
use super::ty::Type;
use crate::name::Name;

pub type Block = Vec<Box<Stmt>>;

pub type Module = Vec<Box<Stmt>>;

/// statement (stmt)
/// stmt -> expr | var_def `;`
#[derive(Debug, PartialEq)]
pub enum StmtKind {
    /// expr_stmt -> expression ';'?
    Expr(Box<Expr>),
    /// ret_stmt -> "ret" expression ';'
    Ret(Box<Expr>),
    /// var_decl -> "var" IDENTIFIER ( '=' expression)? ';'
    Var(Name, Box<Expr>),
    /// block -> '{' declaration* '}'
    Block(Vec<Box<Stmt>>),
    /// func ->
    Func(FuncProto, Block),
    /// extern_func ->
    Extern(FuncProto),
    Impt(Name),
}

#[derive(Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub region: Region,
}

impl Stmt {
    pub fn new(kind: StmtKind, region: Region) -> Box<Self> {
        Box::new(Self { kind, region })
    }
}

/// A function prototype.
/// Captures the function's name and its parameters
#[derive(Debug, PartialEq, Clone)]
pub struct FuncProto {
    pub name: Name,
    pub params: Vec<Param>,
    pub ret: Type,
    pub region: Region,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Param {
    pub name: Name,
    // 'type' is a reserved word in rust :(
    pub ty: Type,
}
