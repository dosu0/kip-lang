//! Abstract Syntax Tree (AST) nodes

/// expression (expr)
/// expr -> number | var | binary | call | var_def
#[derive(Debug, PartialEq)]
pub enum Expr {
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
    Cond(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>),
}

#[derive(Debug, PartialEq)]
pub enum LitKind {
    Int(i64),
    Str(String),
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
#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    /// var_def_stmt -> `var` ident `=` expr `;`
    VarDef(String, Box<Expr>),
    /// ret_stmt -> `ret` expr `;`
    Ret(Box<Expr>),
}

/// A function prototype.
/// Captures the function's name and its parameters
#[derive(Debug, PartialEq)]
pub struct FuncProto {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
}

#[derive(Debug, PartialEq)]
pub struct Param {
    pub name: String,
    // type is a reserved word in rust :(
    pub ty: Type,
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Type {
    // Builtins:
    /// signed and unsigned integers
    Int,
    /// the default return type of a function
    Void,

    /// Could be imported or custom
    Other(String),
}

/// A function definition
/// Captures the function's prototype and its body
/// func_def -> func_proto block
#[derive(Debug, PartialEq)]
pub struct FuncDef {
    pub proto: Box<FuncProto>,
    pub body: Vec<Box<Stmt>>,
}
pub mod visit {
    use super::*;

    pub trait Visitor<T> {
        fn visit_expr(&mut self, e: &Expr) -> T;
        fn visit_stmt(&mut self, s: &Stmt) -> T;
    }

    pub fn walk_expr<T>(v: &mut impl Visitor<T>, e: &Expr) {
        use Expr::*;
        match *e {
            // deadends
            Lit(_) => {},
            Var(_) => {},

            // expressions that may recurse
            Binary(_, _, _) => todo!(),
            Call(_, _) => todo!(),
            Cond(_, _, _) => todo!(),
        }
    }
}
