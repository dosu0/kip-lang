use std::collections::HashMap;

use crate::{
    ast::{FuncDef, FuncProto, Param, Region, Type},
    lexer::Token,
    symbol::Symbol,
};

use super::{ParseResult, Parser};

impl<'a> Parser<'a> {
    fn parse_proto(&mut self) -> ParseResult<Box<FuncProto>> {
        let name = match self.eat() {
            Token::Ident(name) => name,
            _ => return Err(self.syntax_error("expected function name in function prototype")),
        };

        if self.eat() == Token::OpenParen {
            let params = self.parse_param_list()?;
            let ret = if let Token::Colon = self.peek() {
                self.eat();
                self.parse_type_annotation()?
            } else {
                Type::Void
            };

            let proto = FuncProto {
                name: name.clone(),
                params,
                ret,
            };

            // TODO: remove clone
            self.sym_tbl
                .insert(name, Symbol::Func(proto.clone(), HashMap::new()));
            Ok(Box::new(proto))
        } else {
            Err(self.syntax_error("expected `(` in function prototype"))
        }
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        while let Token::Ident(name) = self.eat() {
            if let Token::Colon = self.eat() {
                let ty = self.parse_type_annotation()?;
                params.push(Param { name, ty });

                match self.eat() {
                    // End of argument list
                    Token::CloseParen => break,
                    // More arguments ...
                    Token::Comma => continue,
                    _ => return Err(self.syntax_error("expected `,` or `)` in a parameter list")),
                }
            } else {
                return Err(
                    self.syntax_error("expected a `:` (type annotation) after a parameter name")
                );
            }
        }

        Ok(params)
    }

    pub(super) fn parse_extern(&mut self) -> ParseResult<Box<FuncProto>> {
        if self.eat() != Token::Func {
            return Err(self.syntax_error("expected a function declaration in an extern statement"));
        }

        self.parse_proto()
    }

    pub fn parse_func(&mut self) -> ParseResult<Box<FuncDef>> {
        let mut region = Region {
            start: self.tokens.offset(),
            end: 0,
        };
        let proto = self.parse_proto()?;
        let body = self.parse_block()?;
        self.eat();
        region.end = self.tokens.offset();
        Ok(Box::new(FuncDef {
            proto,
            body,
            region,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinOp, ExprKind::*, StmtKind::*};
    use crate::lexer::TokenStream;
    use crate::source::Source;

    use super::*;

    #[test]
    fn parse_func_def() {
        let input = "\
        func add(x: s32, y: s32): s32 {\n\
            ret x + y;\n\
        }";

        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Token::Func);

        let mut parser = Parser::new(tokens);

        let func = parser.parse_func().unwrap();

        assert_eq!(&*func.proto.name, "add");
        assert_eq!(&*func.proto.params[0].name, "x");
        assert_eq!(func.proto.params[0].ty, Type::Int);
        assert_eq!(&*func.proto.params[1].name, "y");
        assert_eq!(func.proto.params[1].ty, Type::Int);
        assert_eq!(func.proto.ret, Type::Int);

        if let Ret(expr) = &func.body.stmts[0].kind {
            if let Binary(op, lhs, rhs) = &expr.kind {
                assert_eq!(*op, BinOp::Add);
                if let Var(name) = &lhs.kind {
                    assert_eq!(name, "x");
                } else {
                    panic!("expected a variable reference");
                }

                if let Var(name) = &rhs.kind {
                    assert_eq!(name, "y");
                } else {
                    panic!("expected a variable reference");
                }
            }
        } else {
            panic!("expected a return statement");
        }
    }

    #[test]
    fn parse_extern() {
        let input = "extern func raise(sig: s32): s32;";
        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Token::Extern);

        let mut parser = Parser::new(tokens);

        let proto = parser.parse_extern().unwrap();

        assert_eq!(&*proto.name, "raise");
        assert_eq!(&*proto.params[0].name, "sig");
        assert_eq!(proto.params[0].ty, Type::Int);
        assert_eq!(proto.ret, Type::Int);
    }
}
