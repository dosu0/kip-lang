/// A sort of scuffed visitor pattern for the kip AST
/// Because rust enum variants can't be their own types I've had to settle with this
/// TODO: make a macro for all this mess
use super::expr::ExprKind;
use super::lit::Lit;
use super::op::{BinOp, UnOp};
use super::stmt::{Block, FuncProto, StmtKind};
use super::Region;
use super::{expr::Expr, stmt::Stmt};
use crate::name::Name;

pub trait ExprVisitor<T> {
    fn visit_lit_expr(&mut self, lit: Lit, region: Region) -> T;
    fn visit_variable_expr(&mut self, name: Name, region: Region) -> T;
    fn visit_unary_expr(&mut self, op: UnOp, rhs: &Expr, region: Region) -> T;
    fn visit_binary_expr(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, region: Region) -> T;
    fn visit_call_expr(&mut self, func_name: Name, params: &Vec<Box<Expr>>, region: Region) -> T;
    fn visit_cond_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Block,
        else_branch: Option<&Block>,
        region: Region,
    ) -> T;
    fn visit_assign_expr(&mut self, var_name: Name, value: &Expr, region: Region) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_expr_stmt(&mut self, expr: &Expr, region: Region) -> T;
    fn visit_ret_stmt(&mut self, value: &Expr, region: Region) -> T;
    fn visit_var_stmt(&mut self, name: Name, init: &Expr, region: Region) -> T;
    fn visit_block(&mut self, stmts: &Vec<Box<Stmt>>) -> T;
    fn visit_func(&mut self, proto: &FuncProto, body: &Vec<Box<Stmt>>, region: Region) -> T;
    fn visit_extern(&mut self, proto: &FuncProto, region: Region) -> T;
    fn visit_impt(&mut self, symbol: Name, region: Region) -> T;
}

pub fn walk_expr<T>(v: &mut impl ExprVisitor<T>, e: &Expr) -> T {
    match &e.kind {
        ExprKind::Lit(lit) => v.visit_lit_expr(*lit, e.region),
        ExprKind::Variable(sym) => v.visit_variable_expr(*sym, e.region),
        ExprKind::Unary(op, rhs) => v.visit_unary_expr(*op, rhs, e.region),
        ExprKind::Binary(op, lhs, rhs) => v.visit_binary_expr(*op, lhs, rhs, e.region),
        ExprKind::Call(sym, args) => v.visit_call_expr(*sym, args, e.region),
        ExprKind::Cond(condition, then_branch, else_branch) => {
            v.visit_cond_expr(condition, then_branch, else_branch.as_ref(), e.region)
        }
        ExprKind::Assign(variable, expr) => v.visit_assign_expr(*variable, expr, e.region),
    }
}

pub fn walk_stmt<T>(v: &mut impl StmtVisitor<T>, s: &Stmt) -> T {
    match &s.kind {
        StmtKind::Expr(expr) => v.visit_expr_stmt(expr, s.region),
        StmtKind::Ret(value) => v.visit_expr_stmt(value, s.region),
        StmtKind::Var(var, init) => v.visit_var_stmt(*var, init, s.region),
        StmtKind::Block(b) => v.visit_block(b),
        StmtKind::Func(proto, body) => v.visit_func(proto, body, s.region),
        StmtKind::Extern(proto) => v.visit_extern(proto, s.region),
        StmtKind::Impt(symbol) => v.visit_impt(*symbol, s.region),
    }
}
