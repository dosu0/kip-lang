use crate::name::Name;
use std::fmt;

pub use crate::ast::BinOp;
#[derive(Clone, Copy)]
pub enum Instruction {
    /// ex. _L0:
    Label(Name),
    /// ex. _t0 := 4
    Assign(Name, Expr),
    // ex. goto _L1
    Goto(Name),
    /// [`Ifz`]: if-zero
    /// ex. ifz _t0 goto _L0
    Ifz(Primary, Name),
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Primary {
    Const(ConstKind),
    Var(Name),
}

impl fmt::Display for Primary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(name) => name.fmt(f),
            Self::Const(kind) => kind.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConstKind {
    Int(i64),
    Str(Name),
}

impl fmt::Display for ConstKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(f),
            Self::Str(string) => write!(f, "\"{}\"", string.as_str().escape_debug()),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Expr {
    Call(Name),
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
