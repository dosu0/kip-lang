use super::{ParseResult, Parser};
use crate::ast::Type;
use crate::lexer::Token;
use crate::typechk::TypeErrorKind;

impl<'a> Parser<'a> {
    /// type_annotations -> `:` ident
    pub(super) fn parse_type_annotation(&mut self) -> ParseResult<Type> {
        use Type::*;
        use TypeErrorKind::Unknown;
        if let Token::Ident(name) = self.eat() {
            match &*name {
                // builtins
                "s32" => Ok(Int),
                "s64" => Ok(Int),
                "u32" => Ok(Int),
                "u64" => Ok(Int),
                "str" => Ok(Str),

                // types in the symbol table
                s if self.sym_tbl.contains_key(s) => Ok(Other(name)),

                // no such type
                _ => Err(self.type_error(Unknown)),
            }
        } else {
            Err(self.syntax_error("expected type name in type annotation"))
        }
    }
}
