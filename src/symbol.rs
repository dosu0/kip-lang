use std::collections::HashMap;

use crate::ast::{FuncProto, Type};

pub type SymbolTable = HashMap<String, Symbol>;

pub enum Symbol {
    /// proto, local_symbol_table
    Func(FuncProto, SymbolTable),
    Var(Type),
}
