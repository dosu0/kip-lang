//! Abstract Syntax Tree (AST) nodes

/// expression (expr)
/// expr -> lit | var | binary | call | conditional | assign
#[derive(Debug)]
pub enum ExprKind {
    Lit(LitKind),
    /// A variable reference expression, e.g. (`foo`)
    Var(String),
    /// A binary expression, e.g. (`42 + foo`)
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// A function call, e.g. (`some_func(3.14, foo)`)
    /// call_expr -> ident `(` expr* `)`
    #[allow(clippy::vec_box)]
    Call(String, Vec<Box<Expr>>),
    /// an if-else statement
    /// cond_expr -> `if` expr block | `if` expr block `else` block
    Cond(Box<Expr>, Box<Block>, Option<Box<Block>>),
    /// an assignment
    /// assign -> ident `=` expr
    Assign(String, Box<Expr>)
}
/// `Block`: found in conditionals, loops, and function definitions
/// block -> stmt*
#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub region: Region,
}

/// A region of a source file
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum LitKind {
    Int(i64),
    Str(String),
    Char(char)
}

/// ordered from highest to lowest precedence
#[derive(Debug, PartialEq)]
pub enum BinOp {
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Mod,

    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,

    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,

    /// The `==` operator (equality)
    Eq,
    /// The `!=` operator (not equal to)
    Ne,

    /// The `&&` operator (logical and)
    And,
    /// The `||` operator (logical or)
    Or,
}

impl BinOp {
    pub fn get_prec(&self) -> u32 {
        use BinOp::*;

        match *self {
            // Multiplicative (40)
            Mul => 40,
            Div => 40,
            Mod => 40,

            // Additive (20)
            Add => 20,
            Sub => 20,

            // Relational (10)
            Gt => 10,
            Ge => 10,
            Lt => 10,
            Le => 10,

            // Equality (2)
            Eq => 2,
            Ne => 2,

            And => 1,
            Or => 1,
        }
    }
}

/// statement (stmt)
/// stmt -> expr | var_def `;`
#[derive(Debug)]
pub enum StmtKind {
    Expr(Expr),
    /// var_def_stmt -> `var` ident `=` expr `;`
    VarDef(String, Box<Expr>),
    /// ret_stmt -> `ret` expr `;`
    Ret(Box<Expr>),
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub region: Region,
}

/// A function prototype.
/// Captures the function's name and its parameters
#[derive(Debug, PartialEq, Clone)]
pub struct FuncProto {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: String,
    // type is a reserved word in rust :(
    pub ty: Type,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Builtins:
    /// signed and unsigned integers
    Int,
    Str,
    /// the default return type of a function
    Void,
    Bool,

    /// Could be imported or custom
    Other(String),
}

enum Item {
    Func(FuncDef),
}

struct Module {
    items: Vec<Box<Item>>,
}

/// A function definition
/// Captures the function's prototype and its body
/// func_def -> func_proto block
#[derive(Debug)]
pub struct FuncDef {
    pub proto: Box<FuncProto>,
    pub body: Box<Block>,
    pub region: Region,
}

pub mod visit {
    use super::*;

    pub trait Visitor<T> {
        fn visit_expr(&mut self, e: &Expr) -> T;
        fn visit_stmt(&mut self, s: &Stmt) -> T;
        fn visit_func(&mut self, f: &FuncDef) -> T;
        fn visit_block(&mut self, b: &Block) -> T;
    }

    pub fn walk_expr<T>(v: &mut impl Visitor<T>, e: &Expr) {
        use ExprKind::*;
        match e.kind {
            // deadends
            Lit(_) => {}
            Var(_) => {}
            // expressions that may recurse
            Assign(_, ref init) => {
                v.visit_expr(init);
            }
            Binary(_, ref lhs, ref rhs) => {
                v.visit_expr(lhs);
                v.visit_expr(rhs);
            }
            Call(_, ref args) => {
                for arg in args {
                    v.visit_expr(arg);
                }
            }
            Cond(ref condition, _, _) => {
                v.visit_expr(condition);
            },
        }
    }

    pub fn walk_func<T>(v: &mut impl Visitor<T>, f: &FuncDef) {
        v.visit_block(&f.body);
    }
}
