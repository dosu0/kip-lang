//! Type checking
use crate::ast::visit::Visitor as AstVisitor;
use crate::ast::*;

#[derive(Debug, Clone)]
pub enum TypeError {
    Unknown,
}
type TypeChkResult = Result<(), TypeError>;

struct TypeChecker {
    errors: Vec<TypeError>,
}

impl AstVisitor<()> for TypeChecker {
    fn visit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Lit(_) => todo!(),
            Expr::Var(_) => todo!(),
            Expr::Binary(_, _, _h) => todo!(),
            Expr::Call(_, _) => todo!(),
            Expr::Cond(_, _, _) => todo!(),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Expr(e) => self.visit_expr(e),
            Stmt::VarDef(_, _) => todo!(),
            Stmt::Ret(_) => todo!(),
        }
    }
}
