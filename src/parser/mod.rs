//! A top-down parser implementation

pub mod expr;
pub mod func;
pub mod stmt;
pub mod top_lvl;
pub mod ty;

use crate::ast::{FuncDef, FuncProto, Region, Type};
use crate::lexer::{Token, TokenStream};
use crate::source::Source;
use crate::symbol::SymbolTable;
use crate::typechk::TypeErrorKind;
use std::fmt;

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    /// line, col
    loc: (usize, usize),
    kind: ParseErrorKind,
    hint: Option<String>,
}

#[derive(Debug, Clone)]
enum ParseErrorKind {
    SyntaxError(String),
    TypeError(TypeErrorKind),
    RedefinedSymbol(String),
    InvalidModPath(String),
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

        write!(f, " @ line {}, column {}", self.loc.0, self.loc.1)?;

        if let Some(ref hint) = self.hint {
            write!(f, "\nhint: {}", hint)?;
        }

        Ok(())
    }
}

pub struct Parser<'a> {
    sym_tbl: SymbolTable,
    tokens: TokenStream<'a>,
    curr: Token,
    // sort of useless at the moment bc there's no error recovery
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /* fn region_start(&mut self) {}
    fn region_end(&mut self) {} */
    pub fn sym_tbl(self) -> SymbolTable {
        self.sym_tbl
    }

    pub fn input(&self) -> &Source {
        self.tokens.input()
    }

    pub fn new(tokens: TokenStream<'a>) -> Self {
        Self {
            sym_tbl: SymbolTable::new(),
            tokens,
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
            hint: None,
        };
        self.errors.push(err.clone());
        err
    }

    fn redefined_symbol_error(&mut self, sym: &str, hint: Option<&str>) -> ParseError {
        use ParseErrorKind::*;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: RedefinedSymbol(sym.to_owned()),
            hint: hint.map(|h| h.to_owned()),
        };
        self.errors.push(err.clone());
        err
    }

    fn type_error(&mut self, kind: TypeErrorKind) -> ParseError {
        use ParseErrorKind::*;
        let err = ParseError {
            loc: (self.tokens.line(), self.tokens.col()),
            kind: TypeError(kind),
            hint: None,
        };
        self.errors.push(err.clone());
        err
    }

    pub fn parse_top_lvl_expr(&mut self) -> ParseResult<Box<FuncDef>> {
        let mut region = Region {
            start: self.tokens.offset(),
            end: 0,
        };
        let body = vec![self.parse_stmt()?];
        let proto = Box::new(FuncProto {
            name: "__anon_expr".to_owned(),
            params: vec![],
            ret: Type::Void,
        });

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
    use super::*;
    use crate::ast::{BinOp, ExprKind::*, LitKind, StmtKind::*};

    #[test]
    fn parse_paren_expr() {
        let input = "(a + b)";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr().unwrap();

        if let Binary(op, lhs, rhs) = expr.kind {
            assert_eq!(op, BinOp::Add);

            if let Var(name) = lhs.kind {
                assert_eq!(name, "a");
            } else {
                panic!("expected variable reference");
            }

            if let Var(name) = rhs.kind {
                assert_eq!(name, "b");
            } else {
                panic!("expected variable reference");
            }
        } else {
            panic!("expected binary expression");
        }
    }

    #[test]
    fn var_def() {
        use LitKind::*;
        let input = "\
        var my_num: s32 = 42 * 100;\n\
        var my_str: str = \"hello\";";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);

        let var_def = parser.parse_stmt().unwrap();

        if let VarDef(name, init) = var_def.kind {
            assert_eq!(name, "my_num");
            if let Binary(BinOp::Mul, lhs, rhs) = init.kind {
                if let Lit(Int(num)) = lhs.kind {
                    assert_eq!(num, 42);
                } else {
                    panic!("expected a integer literal");
                }

                if let Lit(Int(num)) = rhs.kind {
                    assert_eq!(num, 100);
                } else {
                    panic!("expected a integer literal");
                }
            } else {
                panic!("expected a binary expression");
            }
        } else {
            panic!("expected a variable definition");
        }

        let var_def = parser.parse_stmt().unwrap();

        if let VarDef(name, init) = var_def.kind {
            assert_eq!(name, "my_str");
            if let Lit(Str(str)) = init.kind {
                assert_eq!(str, "hello");
            } else {
                panic!("expected a string literal");
            }
        } else {
            panic!("expected a variable definition");
        }
    }

    #[test]
    #[should_panic]
    fn redefined_symbol_error() {
        let input = "\
        @impt Foo;\n\
        @impt Foo";
        let source = Source::new(input, "<string literal>");
        let tokens = TokenStream::new(&source);
        let mut parser = Parser::new(tokens);
        let item = parser.parse_impt_stmt().unwrap();
        assert_eq!(item, "Foo");
        // should panic
        parser.parse_impt_stmt().unwrap();
    }
}
