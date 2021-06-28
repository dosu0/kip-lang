//! A top-down parser implementation

pub mod expr;
pub mod func;
pub mod stmt;
pub mod top_lvl;

use crate::ast::{FuncDef, FuncProto, Stmt, Type};
use crate::lexer::{Token, TokenStream};
use crate::typechk::TypeError;
use std::collections::HashSet;
use std::fmt;

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    /// line, col
    loc: (usize, usize),
    kind: ParseErrorKind,
}

#[derive(Debug, Clone)]
enum ParseErrorKind {
    SyntaxError(String),
    TypeError(TypeError),
    RedefinedSymbol(String),
    InvalidModPath(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TypeError::*;

        match self {
            Unknown => write!(f, "unknown type"),
        }
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseErrorKind::*;

        match &self.kind {
            SyntaxError(msg) => msg.fmt(f),
            TypeError(e) => e.fmt(f),
            RedefinedSymbol(sym) => write!(f, "{} has already been defined", sym),
            InvalidModPath(path) => write!(f, "cannot find module path \"{}\"", path),
        }?;
        write!(f, " @ line {}, column {}", self.loc.0, self.loc.1)
    }
}

pub struct Parser<'a> {
    sym_tbl: HashSet<String>,
    tokens: TokenStream<'a>,
    curr: Token,
    // sort of useless at the moment bc there's no error recovery
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(input: TokenStream<'a>) -> Self {
        Self {
            sym_tbl: HashSet::new(),
            tokens: input,
            curr: Token::Eof,
            errors: vec![],
        }
    }

    fn eat(&mut self) -> Token {
        self.curr = self.tokens.eat();
        self.curr.clone()
    }

    fn peek(&mut self) -> Token {
        self.tokens.peek()
    }

    fn syntax_error(&mut self, msg: &str) -> ParseError {
        use ParseErrorKind::*;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: SyntaxError(msg.to_string()),
        };
        self.errors.push(err.clone());
        err
    }

    fn redefined_symbol_error(&mut self, sym: &str) -> ParseError {
        use ParseErrorKind::*;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: RedefinedSymbol(sym.to_owned()),
        };
        self.errors.push(err.clone());
        err
    }

    fn type_error(&mut self, kind: TypeError) -> ParseError {
        use ParseErrorKind::*;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: TypeError(kind),
        };
        self.errors.push(err.clone());
        err
    }

    fn parse_var_def(&mut self) -> ParseResult<Box<Stmt>> {
        let name = if let Token::Ident(name) = self.eat() {
            name
        } else {
            return Err(self.syntax_error("expected a variable name in variable declaration"));
        };

        if self.eat() != Token::Equal {
            return Err(self.syntax_error("expected `=` in variable declaration"));
        }

        let init = self.parse_expr()?;

        // symbol hasn't already been defined
        if self.sym_tbl.insert(name.clone()) {
            Ok(Box::new(Stmt::VarDef(name, init)))
        } else {
            Err(self.redefined_symbol_error(&name))
        }
    }

    pub fn parse_top_lvl_expr(&mut self) -> ParseResult<Box<FuncDef>> {
        let body = vec![self.parse_stmt()?];
        let proto = Box::new(FuncProto {
            name: "__anon_expr".to_owned(),
            params: vec![],
            ret: Type::Void,
        });

        Ok(Box::new(FuncDef { proto, body }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, Expr::*, LitKind};

    #[test]
    fn parse_paren_expr() {
        let input = "(a + b)";
        let tokens = TokenStream::new(input);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();

        assert_eq!(
            *expr,
            Binary(
                BinOp::Add,
                Box::new(Var("a".to_owned())),
                Box::new(Var("b".to_owned()))
            ),
        );
    }

    #[test]
    fn var_def() {
        use LitKind::*;
        let input = "\
        var my_num = 42 * 100;\n\
        var my_str = \"hello\";";
        let tokens = TokenStream::new(input);
        let mut parser = Parser::new(tokens);

        let var_def = parser.parse_stmt().unwrap();
        let name = "my_num";
        let init = Box::new(Binary(
            BinOp::Mul,
            Box::new(Lit(Int(42))),
            Box::new(Lit(Int(100))),
        ));
        assert_eq!(*var_def, Stmt::VarDef(name.to_owned(), init));

        let var_def = parser.parse_stmt().unwrap();
        let name = "my_str";
        let init = Box::new(Lit(Str("hello".to_owned())));
        assert_eq!(*var_def, Stmt::VarDef(name.to_owned(), init));
    }

    #[test]
    #[should_panic]
    fn redefined_symbol_error() {
        use LitKind::*;
        let input = "\
        var my_num = 42 * 100;\n\
        var my_num = 69;";
        let tokens = TokenStream::new(input);
        let mut parser = Parser::new(tokens);

        let var_def = parser.parse_stmt().unwrap();
        let name = "my_num";
        let init = Box::new(Binary(
            BinOp::Mul,
            Box::new(Lit(Int(42))),
            Box::new(Lit(Int(100))),
        ));
        assert_eq!(*var_def, Stmt::VarDef(name.to_owned(), init));

        // should panic
        parser.parse_stmt().unwrap();
    }
}
