use crate::name::Name;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    // Primitve Types
    /// Integer types
    /// examples: 'u8', 's16', 'u32', 's64'
    Int { signed: bool, size: IntSize },
    /// Boolean type (true and false)
    Bool,
    /// 'void' (nothing/empty)
    Void,

    /// Other types
    Name(Name),
}

impl Type {
    // Returns true if the type is a primitive type
    // Primitve types include void, integers, and booleans
    fn is_primitive(&self) -> bool {
        !matches!(self, Self::Name(_))
    }
}

/// can be 8, 16, 32, and 64
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntSize(u8);

impl IntSize {
    pub fn new(size: u8) -> Self {
        if [8, 16, 32, 64].contains(&size) {
            Self(size)
        } else {
            panic!("invalid integer size")
        }
    }
}
