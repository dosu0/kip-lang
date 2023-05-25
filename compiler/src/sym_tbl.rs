use crate::ast::stmt::FuncProto;
use crate::ast::Type;
use crate::name::Name;

use std::collections::HashMap;
use std::rc::Rc;

pub struct SymbolTable {
    // TODO: change into Rc
    enclosing: Option<Rc<SymbolTable>>,
    values: HashMap<Name, SymbolType>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: &Rc<SymbolTable>) -> SymbolTable {
        SymbolTable {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: Name, sym: SymbolType) {
        self.values.insert(name, sym);
    }

    pub fn lookup(&self, name: Name) -> Option<&SymbolType> {
        // first check if the symbol exists in this table
        if self.values.contains_key(&name) {
            return self.values.get(&name);
        }

        // then check if it exists in higher tables
        self.enclosing.as_ref().and_then(|e| e.lookup(name))
    }
}

pub enum SymbolType {
    Var(Option<Type>),
    Func(FuncProto),
    Impt,
}

impl SymbolType {
    pub fn as_var(&self) -> Option<&Type> {
        if let Self::Var(v) = self {
            v.as_ref()
        } else {
            None
        }
    }

    /// Returns `true` if the symbol is [`Var`].
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(..))
    }
}
