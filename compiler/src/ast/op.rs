use std::fmt;

/// ordered from highest to lowest precedence
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
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

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mul => '*'.fmt(f),
            Self::Div => '/'.fmt(f),
            Self::Mod => '%'.fmt(f),
            Self::Add => '+'.fmt(f),
            Self::Sub => '-'.fmt(f),
            Self::Ge => ">=".fmt(f),
            Self::Gt => '>'.fmt(f),
            Self::Lt => '<'.fmt(f),
            Self::Le => "<=".fmt(f),
            Self::Eq => "==".fmt(f),
            Self::Ne => "!=".fmt(f),
            Self::And => "&&".fmt(f),
            Self::Or => "||".fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnOp {
    Not,
    Neg,
    // TODO:
    //  Move,
    //  Clone
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Not => '!'.fmt(f),
            Self::Neg => '-'.fmt(f),
        }
    }
}
