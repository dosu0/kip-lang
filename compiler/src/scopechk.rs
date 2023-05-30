use crate::ast::BinOp;
use crate::ast::Block;
use crate::ast::Lit;
use crate::ast::Region;
use crate::ast::UnOp;
use std::collections::HashMap;

use crate::ast::Expr;
use crate::ast::stmt::{FuncProto, Module, Stmt};
use crate::ast::visit::{walk_expr, walk_stmt, StmtVisitor};
use crate::ast::ExprVisitor;
use crate::name::Name as Symbol;

type Scope = HashMap<Symbol, bool>;

pub struct ScopeChecker {
    /// a stack of accesible scopes
    /// each scope is a hashmap where the keys are the symbol names and the values store whether or
    /// not they are initialized
    scopes: Vec<Scope>,
}

// FIXME: This current implementation has a lot of memory overhead
// FIXME: Return errors instead of just printing them
impl ScopeChecker {
    pub fn new() -> ScopeChecker {
        Self {
            // init the global scope
            scopes: vec![Scope::new()], // sym_tbl: SymbolTable::new(),
        }
    }

    pub fn check(&mut self, module: &Module) {
        for stmt in module {
            walk_stmt(self, &stmt)
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr)
    }

    fn start_scope(&mut self) {
        self.scopes.push(Scope::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Symbol) {
        if self.current_scope().contains_key(&name) {
            eprintln!("variable `{}` already exists in this scope", name);
        } else {
            self.current_scope_mut().insert(name, false);
        }
    }

    fn define(&mut self, name: Symbol) {
        self.current_scope_mut().insert(name, true);
    }

    fn current_scope(&self) -> &Scope {
        // NOTE: this unwrap is fine because there should always be a global scope
        return self.scopes.first().unwrap();
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        // NOTE: this unwrap is fine because there should always be a global scope
        return self.scopes.first_mut().unwrap();
    }

    fn check_local(&mut self, sym: Symbol) {
        // go thru every scope, starting from the innermost
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&sym) {
                return;
            }
        }

        eprintln!("'{}' is not defined", sym)
    }

    fn check_function(&mut self, proto: &FuncProto, body: &Block) {
        self.start_scope();

        // bind each parameter to a local variable in the function
        for param in &proto.params {
            self.declare(param.name);
            self.define(param.name);
        }
        
        self.check(body);

        self.end_scope();
    }
}

impl StmtVisitor<()> for ScopeChecker {
    fn visit_expr_stmt(&mut self, expr: &Expr, _: Region) {
        walk_expr(self, &expr)
    }

    fn visit_ret_stmt(&mut self, value: &Expr, _: Region) {
        self.check_expr(value);
    }

    fn visit_var_stmt(&mut self, name: Symbol, init: &Expr, _: Region) {
        self.declare(name);
        // check the initializer before we define the variable
        self.check_expr(&init);
        self.define(name);
    }

    fn visit_func(&mut self, proto: &FuncProto, body: &Block, _: Region) {
        self.declare(proto.name);
        self.define(proto.name);

        self.check_function(proto, body);
    }

    fn visit_extern(&mut self, proto: &FuncProto, _: Region) {
        // extern just declares this function
        // its the linkers job to actually go and find this function
        self.declare(proto.name);
        self.define(proto.name);
    }

    fn visit_impt(&mut self, _: Symbol, _: Region) {
        todo!()
    }

    fn visit_block(&mut self, stmts: &Vec<Box<Stmt>>)  {
        self.start_scope();
        self.check(stmts);
        self.end_scope();
    }
}

impl ExprVisitor<()> for ScopeChecker {
    fn visit_lit_expr(&mut self, _: Lit, _: Region) {
    }

    fn visit_variable_expr(&mut self, var: Symbol, _: Region) {
        let initialized = self.current_scope().get(&var).unwrap_or(&true);
        if !initialized {
            eprintln!("cannot reference variable in its own initializer");
        }

        self.check_local(var);
    }

    fn visit_unary_expr(&mut self, _: UnOp, rhs: &Expr, _: Region) {
        self.check_expr(rhs);
    }

    fn visit_binary_expr(&mut self, _: BinOp, lhs: &Expr, rhs: &Expr, _: Region) -> () {
        self.check_expr(lhs);
        self.check_expr(rhs);
    }

    fn visit_call_expr(&mut self, func_name: Symbol, args: &Vec<Box<Expr>>, _: Region) {
        self.check_local(func_name);

        for arg in args {
            self.check_expr(arg);
        }
    }

    fn visit_cond_expr(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        _: Region,
    ) {
        self.check_expr(condition);
        self.check(then_block);

        if let Some(else_block) = else_block {
            self.check(else_block);
        }
    }

    fn visit_assign_expr(&mut self, var_name: Symbol, value: &Expr, _: Region) {
        // scope check the value
        self.check_expr(value);
        // check that this variable exists
        self.check_local(var_name);
    }
}
