use crate::name::Name as Symbol;
use std::fmt;

pub use crate::ast::BinOp;
#[derive(Clone, Copy)]
pub enum Instruction {
    /// ex. _L0:
    Label(Symbol),
    /// ex. _t0 := 4
    Assign(Symbol, Expr),
    // ex. goto _L1
    Goto(Symbol),
    /// [`Ifz`]: if-zero
    /// ex. ifz _t0 goto _L0
    Ifz(Primary, Symbol),
    /// ex. arg x
    Arg(Primary),
    /// ex. ret x
    Ret(Option<Primary>),
}

impl Instruction {
    /// Returns `true` if the instruction is [`Ifz`].
    pub fn is_ifz(&self) -> bool {
        matches!(self, Self::Ifz(..))
    }

    /// Returns `true` if the instruction is [`Label`].
    pub fn is_label(&self) -> bool {
        matches!(self, Self::Label(..))
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Label(name) => write!(f, "{}:", name),
            Self::Assign(var, init) => write!(f, "{} := {}", var, init),
            Self::Goto(label) => write!(f, "goto {}", label),
            Self::Ifz(value, label) => write!(f, "ifz {} goto {}", value, label),
            Self::Arg(value) => write!(f, "arg {}", value),
            Self::Ret(value) => {
                if let Some(value) = value {
                    write!(f, "ret {}", value)
                } else {
                    write!(f, "ret")
                }
            }
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Primary {
    Const(ConstKind),
    Var(Symbol)
}

impl fmt::Display for Primary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(name) => name.fmt(f),
            Self::Const(kind) => kind.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConstKind {
    Int(i64),
    Str(Symbol),
}

impl fmt::Display for ConstKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(f),
            Self::Str(string) => write!(f, "\"{}\"", string.as_str().escape_debug()),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Expr {
    Call(Symbol),
    Binary(BinOp, Primary, Primary),
    Primary(Primary),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(op, lhs, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
            Self::Primary(value) => write!(f, "{}", value),
            Self::Call(name) => write!(f, "call {}", name),
        }
    }
}
