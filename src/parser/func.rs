use crate::{
    ast::{FuncDef, FuncProto, Param, Type},
    lexer::Token,
};

use super::{ParseResult, Parser, TypeError};

impl<'a> Parser<'a> {
    fn parse_proto(&mut self) -> ParseResult<Box<FuncProto>> {
        use TypeError::Unknown;

        let name = match self.eat() {
            Token::Ident(name) => name,
            _ => return Err(self.syntax_error("expected function name in function prototype")),
        };

        if self.eat() == Token::OpenParen {
            let params = self.parse_param_list()?;
            let ret = if self.peek() == Token::Colon {
                self.eat();
                if let Token::Ident(name) = self.eat() {
                    match &*name {
                        "s32" => Type::Int,
                        "s64" => Type::Int,
                        "u32" => Type::Int,
                        "u64" => Type::Int,
                        s if self.sym_tbl.contains(s) => Type::Other(name),
                        _ => return Err(self.type_error(Unknown)),
                    }
                } else {
                    return Err(
                        self.syntax_error("expected type name after `:` in function prototype")
                    );
                }
            } else {
                Type::Void
            };

            let proto = FuncProto { name, params, ret };
            Ok(Box::new(proto))
        } else {
            Err(self.syntax_error("expected `(` in function prototype"))
        }
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        use TypeError::Unknown;
        let mut params = Vec::new();

        while let Token::Ident(name) = self.eat() {
            if let Token::Colon = self.eat() {
                let ty = if let Token::Ident(name) = self.eat() {
                    match &*name {
                        "s32" => Type::Int,
                        "s64" => Type::Int,
                        "u32" => Type::Int,
                        "u64" => Type::Int,
                        _ => return Err(self.type_error(Unknown)),
                    }
                } else {
                    return Err(self.syntax_error("expected type name in type annotation"));
                };

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

    pub(super) fn parse_func(&mut self) -> ParseResult<Box<FuncDef>> {
        let proto = self.parse_proto()?;

        if self.eat() != Token::OpenBrace {
            return Err(self.syntax_error("expected `{` in function definition"));
        }

        let mut body = Vec::new();

        while self.peek() != Token::CloseBrace {
            let stmt = self.parse_stmt()?;
            body.push(stmt);
        }

        self.eat();

        Ok(Box::new(FuncDef { proto, body }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinOp, Expr::*, Stmt::*};
    use crate::lexer::TokenStream;

    use super::*;

    #[test]
    fn parse_func_def() {
        let input = "\
        func add(x: s32, y: s32) {\n\
            var sum = x + y;\n\
            ret sum;\n\
        }";

        let mut tokens = TokenStream::new(input);
        assert_eq!(tokens.eat(), Token::Func);

        let mut parser = Parser::new(tokens);

        let func = parser.parse_func().unwrap();

        assert_eq!(&*func.proto.name, "add");
        assert_eq!(&*func.proto.params[0].name, "x");
        assert_eq!(func.proto.params[0].ty, Type::Int);
        assert_eq!(&*func.proto.params[1].name, "y");
        assert_eq!(func.proto.params[1].ty, Type::Int);
        assert_eq!(func.proto.ret, Type::Void);

        let binary_expr = Box::new(Binary(
            BinOp::Add,
            Box::new(Var("x".to_owned())),
            Box::new(Var("y".to_owned())),
        ));
        assert_eq!(*func.body[0], VarDef("sum".to_owned(), binary_expr));
        assert_eq!(*func.body[1], Ret(Box::new(Var("sum".to_owned()))));
    }

    #[test]
    fn parse_extern() {
        let input = "extern func raise(sig: s32): s32;";

        let mut tokens = TokenStream::new(input);
        assert_eq!(tokens.eat(), Token::Extern);

        let mut parser = Parser::new(tokens);

        let proto = parser.parse_extern().unwrap();

        assert_eq!(&*proto.name, "raise");
        assert_eq!(&*proto.params[0].name, "sig");
        assert_eq!(proto.params[0].ty, Type::Int);
        assert_eq!(proto.ret, Type::Int);
    }
}
