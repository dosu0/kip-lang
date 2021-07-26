//! Type checking
use crate::ast::visit::{walk_expr, Visitor as AstVisitor};
use crate::ast::*;
use crate::source::Source;
use crate::symbol::{Symbol, SymbolTable};
use log::error;
use std::{fmt, io};
use TypeErrorKind::*;

#[derive(Debug)]
pub struct TypeError<'a> {
    kind: TypeErrorKind,
    source: &'a Source,
    region: Region,
}

impl<'a> TypeError<'a> {
    pub fn display(&self) -> io::Result<()> {
        let len = self.region.end - self.region.start;

        error!(target: "type_checker", "{}: {}", self.source.name, self.kind);

        let string = self
            .source
            .contents
            .chars()
            .skip(self.region.start)
            .take(len)
            .collect::<String>();

        eprintln!("here:");
        eprintln!("{}", string);
        for _ in 0..len {
            eprint!("^");
        }

        eprintln!();

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    Unknown,
    UndefinedSymbol,
    UndefinedFunc,
    InvalidArgs,
    CannotInfer,
    /// type mismatch
    Mismatch,
    /// invalid operation on a type
    InvalidOp,
    /// the return type doesn't match the type in the function prototype
    RetTyMismatch,
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unknown => "unknown type".fmt(f),
            UndefinedSymbol => "undefined symbol".fmt(f),
            UndefinedFunc => "undefined function; hint: try defining or importing it".fmt(f),
            InvalidArgs => "invalid arguments".fmt(f),
            CannotInfer => "cannot infer type".fmt(f),
            Mismatch => "type mismatch".fmt(f),
            InvalidOp => "invalid operation".fmt(f),
            RetTyMismatch => "the return type mismatches the type in the function prototype".fmt(f),
        }
    }
}

// type TypeChkResult = Result<(), TypeError>;

pub struct TypeChecker<'a, 'b> {
    input: &'a Source,
    sym_tbl: &'b SymbolTable,
    local_scope: &'b SymbolTable,
    errors: Vec<TypeError<'a>>,
}

impl<'a, 'b> TypeChecker<'a, 'b> {
    pub fn new(input: &'a Source, sym_tbl: &'b SymbolTable) -> Self {
        Self {
            errors: vec![],
            sym_tbl,
            local_scope: sym_tbl,
            input,
        }
    }

    pub fn get_var_type(&self, name: &str) -> Option<&Type> {
        self.local_scope
            .get(name)
            .and_then(|sym| sym.as_var())
            .or_else(|| self.sym_tbl.get(name).and_then(|sym| sym.as_var()))
    }

    pub fn var_exists(&self, name: &str) -> bool {
        self.local_scope
            .get(name)
            .map(|sym| sym.is_var())
            .unwrap_or_else(|| self.sym_tbl.get(name).map_or(false, |sym| sym.is_var()))
    }

    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    fn type_error(&mut self, kind: TypeErrorKind, region: Region) {
        self.errors.push(TypeError {
            kind,
            region,
            source: self.input,
        })
    }

    fn infer_type(&self, e: &Expr) -> Option<Type> {
        use ExprKind::*;
        use LitKind::*;
        match &e.kind {
            Lit(kind) => match kind {
                Int(_) | Char(_) => Some(Type::Int),
                Str(_) => Some(Type::Str),
            },

            Assign(ref name, _) => match self.sym_tbl.get(name) {
                // TODO: remove this clone?
                Some(Symbol::Var(ty)) => Some(ty.clone()),
                _ => None,
            },
            Var(ref name) => self.get_var_type(name).map(|t| t.clone()),
            Binary(_, ref lhs, ref rhs) => match (self.infer_type(lhs), self.infer_type(rhs)) {
                (Some(lty), Some(rty)) if lty == rty => Some(lty),
                _ => None,
            },
            Call(ref name, _) => match self.sym_tbl.get(name) {
                // TODO: remove this clone?
                Some(Symbol::Func(proto, _)) => Some(proto.ret.clone()),
                _ => None,
            },
            Cond(_, _, _) => todo!(),
        }
    }
}

impl<'a, 'b> AstVisitor<()> for TypeChecker<'a, 'b> {
    fn visit_expr(&mut self, e: &Expr) {
        use ExprKind::*;
        use TypeErrorKind::*;
        match &e.kind {
            Lit(_) => {}

            Assign(ref name, _) => {
                if !self.var_exists(name) {
                    self.type_error(UndefinedSymbol, e.region);
                }
            }

            Var(ref name) => {
                if !self.var_exists(name) {
                    self.type_error(UndefinedSymbol, e.region);
                }
            }

            Binary(op, ref lhs, ref rhs) => {
                use BinOp::*;

                self.visit_expr(lhs);
                self.visit_expr(rhs);

                let ty = match (self.infer_type(lhs), self.infer_type(rhs)) {
                    (Some(lty), Some(rty)) if lty == rty => lty,
                    _ => return self.type_error(Mismatch, e.region),
                };

                match ty {
                    Type::Str if *op != Eq || *op != Ne => self.type_error(InvalidOp, e.region),
                    Type::Other(_) | Type::Void => self.type_error(InvalidOp, e.region),
                    _ => {}
                }
            }

            Call(ref name, args) => {
                use Symbol::*;

                let params = match self.sym_tbl.get(name) {
                    Some(Func(proto, _)) => &proto.params,
                    _ => return self.type_error(UndefinedFunc, e.region),
                };

                if args.len() != params.len() {
                    return self.type_error(InvalidArgs, e.region);
                }

                for (arg, param) in args.iter().zip(params) {
                    self.visit_expr(arg);
                    if let Some(arg_ty) = self.infer_type(arg) {
                        if arg_ty != param.ty {
                            self.type_error(InvalidArgs, e.region);
                        }
                    }
                }
            }

            Cond(_, _, _) => {
                walk_expr(self, e);
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        use StmtKind::*;
        match &s.kind {
            Expr(e) => self.visit_expr(e),
            VarDef(name, init) => {
                self.visit_expr(init);
                if let Some(inferred_ty) = self.infer_type(init) {
                    if let Some(ty) = self.get_var_type(name) {
                        if *ty != inferred_ty {
                            self.type_error(Mismatch, s.region);
                        }
                    } else {
                        self.type_error(UndefinedSymbol, s.region);
                    }
                } else {
                    self.type_error(CannotInfer, s.region);
                }
            }
            Ret(_) => {}
        }
    }

    fn visit_func(&mut self, f: &FuncDef) {
        use StmtKind::Ret;

        if let Some(Symbol::Func(_, local)) = self.sym_tbl.get(&f.proto.name) {
            self.local_scope = local;
        } else {
            self.type_error(UndefinedFunc, f.region);
        }

        for stmt in &f.body.stmts {
            match &stmt.kind {
                Ret(e) => {
                    self.visit_expr(e);
                    if let Some(ty) = self.infer_type(e) {
                        if ty != f.proto.ret {
                            self.type_error(RetTyMismatch, f.region);
                        }
                    } else {
                        self.type_error(CannotInfer, f.region);
                    }
                }
                _ => self.visit_stmt(stmt),
            }
        }
    }

    fn visit_block(&mut self, b: &Block) {
        for stmt in &b.stmts {
            self.visit_stmt(stmt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Token, TokenStream};
    use crate::parser::Parser;

    use super::*;

    #[test]
    fn typechk_func() {
        let input = "\
        func ret_ty_mismatch(): s32 {\n\
            ret \"str\";\n\
        }";

        let source = Source::new(input, "<string literal>");
        let mut tokens = TokenStream::new(&source);
        assert_eq!(tokens.eat(), Token::Func);

        let mut parser = Parser::new(tokens);
        let f = parser.parse_func().unwrap();

        let sym_tbl = parser.sym_tbl();
        let mut typechk = TypeChecker::new(&source, sym_tbl);
        typechk.visit_func(&f);

        let e = typechk.errors().last().unwrap();
        assert!(matches!(e.kind, TypeErrorKind::RetTyMismatch))
    }
}
