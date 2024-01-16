use std::fmt;
use crate::name::Name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lit {
    Int(i64),
    Str(Name),
    Char(char),
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(num) => num.fmt(f),
            Self::Str(string) => write!(f, "\"{}\"", string),
            Self::Char(c) => c.fmt(f),
        }
    }
}

impl From<i64> for Lit {
    fn from(int: i64) -> Self {
        Self::Int(int)
    }
}

impl From<&str> for Lit {
    fn from(string: &str) -> Self {
        Self::Str(Name::from(string))
    }
}

impl From<Name> for Lit {
    fn from(name: Name) -> Self {
        Self::Str(name)
    }
}

impl From<char> for Lit {
    fn from(c: char) -> Self {
        Self::Char(c)
    }
}
