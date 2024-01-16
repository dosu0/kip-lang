use super::Parser;
use crate::ast::ty::IntSize;
use crate::ast::Type;
// use crate::interner::sym;
use crate::name::*;

use anyhow::Result;

impl Parser {
    /// type_annotation -> `:` ident
    pub(super) fn type_annotation(&mut self) -> Result<Type> {
        let type_name = self.expect_ident("expected a type name")?;

        if type_name == name("bool") {
            return Ok(Type::Bool);
        }

        let (size, signed) = match type_name {
            t if t == name("uint8") => (8, false),
            t if t == name("uint16") => (16, false),
            t if t == name("uint32") => (32, false),
            t if t == name("uint64") => (64, false),
            t if t == name("int8") => (8, true),
            t if t == name("int16") => (16, true),
            t if t == name("int32") => (32, true),
            t if t == name("int64") => (64, true),
            _ => return Ok(Type::Name(type_name)),
        };

        Ok(Type::Int {
            signed,
            size: IntSize::new(size),
        })
    }
}
