use std::collections::HashMap;

use crate::ast::{FuncProto, Type};

pub type SymbolTable = HashMap<String, Symbol>;

pub enum Symbol {
    /// prototype, local_scope
    Func(FuncProto, SymbolTable),
    Var(Type),
}

impl Symbol {
    pub fn as_var(&self) -> Option<&Type> {
        if let Self::Var(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol is [`Var`].
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(..))
    }
}
