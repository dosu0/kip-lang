use super::Parser;

use crate::ast::lit::Lit;
use crate::ast::stmt::{FuncProto, Param, Stmt, StmtKind};
use crate::ast::ty::Type;
use crate::token::TokenKind::*;

use anyhow::{anyhow, Result};

type StmtResult = Result<Box<Stmt>>;

impl Parser {
    // NOTE: might rename to definition
    pub(super) fn declaration(&mut self) -> StmtResult {
        let decl = match self.peek().kind {
            Extern => self.extern_decl(),
            Func => self.func_decl(),
            Var => self.var_decl(),
            Impt => self.impt(),
            _ => self.statement(),
        };

        if let Err(ref _parse_error) = decl {
            self.sync();
            // self.errors.push(parse_error);
        }

        decl
    }

    fn extern_decl(&mut self) -> StmtResult {
        // eat 'extern'
        let extern_kw = self.eat();
        self.expect(Func, "expected 'func'")?;
        let proto = self.proto()?;
        let semicolon = self.expect(Semicolon, "expected ';' after function prototype")?;
        Ok(Stmt::new(
            StmtKind::Extern(proto),
            self.region_from(extern_kw, semicolon),
        ))
    }

    // Parse function declarations
    fn func_decl(&mut self) -> StmtResult {
        // eat 'func'
        let func_kw = self.eat();
        let proto = self.proto()?;
        self.expect(OpenBrace, "expected '{' before function body")?;
        let body = self.block()?;
        let close_brace = self.previous();

        Ok(Stmt::new(
            StmtKind::Func(proto, body),
            self.region_from(func_kw, close_brace),
        ))
    }

    // helper function to parse punction prototypes
    fn proto(&mut self) -> Result<FuncProto> {
        // note where the function prototype starts
        let proto_start = self.peek();
        // get the function name
        let name = self.expect_ident("expected function name")?;
        self.expect(OpenParen, "expected '(' after function name")?;

        let mut params = Vec::new();

        // check if there is a parameter list
        if !self.check(CloseParen) {
            loop {
                // eat the parameter name
                let name = self.expect_ident("expected parameter list")?;
                self.expect(Colon, "expected type annotation after parameter name")?;
                let ty = self.type_annotation()?;
                params.push(Param { ty, name });

                if !self.matches(&[Comma]) {
                    break;
                }
            }
        }

        let close_paren = self.expect(CloseParen, "expected ')' after parameter list")?;

        let ret = if self.matches(&[Colon]) {
            self.type_annotation()?
        } else {
            Type::Void
        };

        Ok(FuncProto {
            name,
            params,
            ret,
            region: self.region_from(proto_start, close_paren), //
        })
    }

    fn statement(&mut self) -> StmtResult {
        match self.peek().kind {
            Ret => self.ret_stmt(),
            OpenBrace => {
                let open_brace = self.eat();
                Ok(Stmt::new(
                    StmtKind::Block(self.block()?),
                    self.region_since(open_brace), // the block's region is from the '{' -> '}'
                ))
            }
            Impt => self.impt(),
            _ => self.expr_stmt(),
        }
    }

    // parses return statements
    fn ret_stmt(&mut self) -> StmtResult {
        // eat 'ret'
        let ret_kw = self.eat();
        let value = self.expression()?;
        let semicolon = self.expect(Semicolon, "expected ';' at end of return statement")?;
        Ok(Stmt::new(
            StmtKind::Ret(value),
            self.region_from(ret_kw, semicolon),
        ))
    }

    pub fn block(&mut self) -> Result<Vec<Box<Stmt>>> {
        assert_eq!(self.previous(), OpenBrace);
        let mut stmts = Vec::new();
        while !self.check(CloseBrace) && !self.is_eof() {
            stmts.push(self.declaration()?);
        }

        // eat '}'
        self.expect(CloseBrace, "expected '}' to close block")?;

        Ok(stmts)
    }

    // parses statements that could also be considered as an expression
    fn expr_stmt(&mut self) -> StmtResult {
        let expr_start = self.peek();
        let expr = self.expression()?;

        // if statements dont require a semicolon, but every other one does
        if !expr.is_cond() {
            self.expect(Semicolon, "expected semicolon after expression")?;
        }

        Ok(Stmt::new(
            StmtKind::Expr(expr),
            self.region_since(expr_start),
        ))
    }

    /// TODO: make the initializer optional
    fn var_decl(&mut self) -> StmtResult {
        // eat 'var'
        let var_kw = self.eat();
        let name = self.expect_ident("expected variable name")?;

        self.expect(Equal, "expected '=' in variable declaration")?;

        let init = self.expression()?;

        // eat ';'
        let semicolon = self.expect(Semicolon, "expected ';' at end of variable declaration")?;

        Ok(Stmt::new(
            StmtKind::Var(name, init),
            self.region_from(var_kw, semicolon),
        ))
    }

    fn impt(&mut self) -> StmtResult {
        use Lit::Str;

        // eat '@impt'
        let impt_kw = self.eat();

        if let Literal(Str(path)) = self.peek().kind {
            let path_lit = self.eat();
            Ok(Stmt::new(
                StmtKind::Impt(path),
                self.region_from(impt_kw, path_lit),
            ))
        } else {
            Err(anyhow!(
                "expected path after 'impt' statement {}",
                self.peek(),
            ))
        }
    }
}
