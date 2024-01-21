//! Visitor pattern for the kip intermediate code
//!
//!

use crate::name::Name;

use super::ic::{Expr, Primary};

pub trait InstructionVistor<T> {
    fn visit_label(&mut self, name: Name) -> T;
    fn visit_assign(&mut self, variable: Name, value: Expr) -> T;
    fn visit_goto(&mut self, label: Name) -> T;
    fn visit_ifz(&mut self, condition: Primary, label: Name) -> T;
    fn visit_arg(&mut self, arg: Primary) -> T;
    fn visit_ret(&mut self, value: Option<Primary>) -> T;
}
