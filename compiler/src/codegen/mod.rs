//! `codegen`: Generates TAC IR from a Kip AST. Or in other words, this module generates an
//! intermediate representation of kip source code in three-address code. Three-address code is
//! high-level enough that it's machine indepedent, but low level enough that it's easy (enough) to
//! convert to assembly/machine code.

use std::collections::HashMap;

use crate::ast::Lit;

use crate::ast::visit::ExprVisitor;
use crate::ast::visit::StmtVisitor;
use crate::ast::visit::{walk_expr, walk_stmt};
use crate::ast::stmt::{FuncProto, Module, Stmt};
use crate::ast::stmt::Block;
use crate::ast::BinOp;
use crate::ast::Expr;
use crate::ast::Region;
use crate::ast::UnOp;
use crate::name::Name as Symbol;
use crate::name::name;

use std::fmt::Write;

/// [`ic`]: intermediate code
pub mod ic;

fn blocks(instructions: &mut Vec<ic::Instruction>) -> Vec<&mut [ic::Instruction]> {
    instructions.split_inclusive_mut(|i| i.is_ifz()).collect()
}

fn elim_common_subexprs(block: &mut [ic::Instruction]) {
    use ic::Instruction::Assign;
    use ic::{Expr, Primary};

    // list of the avaible expressions
    let mut available_expressions: HashMap<Symbol, Expr> = HashMap::new();

    for instruction in block {
        todo!()
    }
}

#[derive(Default)]
pub struct CodeGenerator {
    instructions: Vec<ic::Instruction>,
    // the number of temporary variables the code generator has created
    tmp_var: usize,
    // the number of temporary labels the code generator has created
    tmp_label: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn gen(&mut self, module: &Module) {
        for stmt in module {
            walk_stmt(self, &stmt);
        }
    }
    // returns a Symbol in the form of "_t{number}" to store temporary values
    fn new_tmp_var(&mut self) -> Symbol {
        let tmp_var_name = format!("_t{}", self.tmp_var);
        self.tmp_var += 1;
        name(&tmp_var_name)
    }

    // returns a Symbol in the form of "_t{number}" for store temporary values
    fn new_tmp_label(&mut self) -> Symbol {
        let label_name = format!("_L{}", self.tmp_label);
        self.tmp_label += 1;
        name(&label_name)
    }

    // wrapper types suck fr

    fn emit_assign_const_int(&mut self, name: Symbol, init: i64) {
        self.emit_assign(
            name,
            ic::Expr::Primary(ic::Primary::Const(ic::ConstKind::Int(init))),
        );
    }

    fn emit_assign_const_str(&mut self, name: Symbol, init: Symbol) {
        self.emit_assign(
            name,
            ic::Expr::Primary(ic::Primary::Const(ic::ConstKind::Str(init))),
        );
    }

    fn emit_assign_var(&mut self, name: Symbol, init: Symbol) {
        self.emit_assign(name, ic::Expr::Primary(ic::Primary::Var(init)));
    }

    fn emit_assign(&mut self, name: Symbol, init: ic::Expr) {
        self.instructions.push(ic::Instruction::Assign(name, init));
    }

    fn emit_assign_call(&mut self, name: Symbol, func: Symbol) {
        self.instructions
            .push(ic::Instruction::Assign(name, ic::Expr::Call(func)));
    }

    fn emit_assign_binary(&mut self, name: Symbol, op: ic::BinOp, lhs: Symbol, rhs: Symbol) {
        self.instructions.push(ic::Instruction::Assign(
            name,
            ic::Expr::Binary(op, ic::Primary::Var(lhs), ic::Primary::Var(rhs)),
        ));
    }
    fn emit_arg(&mut self, name: Symbol) {
        self.instructions
            .push(ic::Instruction::Arg(ic::Primary::Var(name)));
    }

    fn emit_ifz(&mut self, condition: Symbol, label: Symbol) {
        self.instructions
            .push(ic::Instruction::Ifz(ic::Primary::Var(condition), label));
    }

    fn emit_label(&mut self, label: Symbol) {
        self.instructions.push(ic::Instruction::Label(label));
    }

    fn emit_ret(&mut self, value: Option<Symbol>) {
        self.instructions
            .push(ic::Instruction::Ret(value.map(ic::Primary::Var)));
    }

    pub fn get_intermediate_code(&mut self) -> String {
        let mut ic = String::new();
        for instruction in &self.instructions {
            writeln!(&mut ic, "{}", instruction).unwrap();
        }

/*
        self.optimize();

        println!("; optimized instructions");
        for instruction in &self.instructions {
            println!("{}", instruction);
        }
*/
        ic
    }

    pub fn optimize(&mut self) {
        for block in blocks(&mut self.instructions) {
            elim_common_subexprs(block);
        }
    }
}
/// Only [`CodeGenerator::visit_expr`] returns a string (the name of temporary it generates)
impl StmtVisitor<Option<Symbol>> for CodeGenerator {

    fn visit_expr_stmt(&mut self, expr: &Expr, _: Region) -> Option<Symbol> {
        walk_expr(self, &expr)
    }

    fn visit_ret_stmt(&mut self, value: &Expr, _: Region) -> Option<Symbol> {
        let t = walk_expr(self, value);
        self.emit_ret(t);
        None
    }

    fn visit_var_stmt(&mut self, name: Symbol, init: &Expr, _: Region) -> Option<Symbol> {
        let initializer_symbol = walk_expr(self, init).unwrap();
        self.emit_assign_var(name, initializer_symbol);
        None
    }

    fn visit_func(&mut self, proto: &FuncProto, body: &Vec<Box<Stmt>>, _: Region) -> Option<Symbol> {
        self.emit_label(proto.name);
        self.visit_block(body)
    }

    // TODO: implement something
    fn visit_extern(&mut self, proto: &FuncProto, _: Region) -> Option<Symbol> {
        None
    }

    // TODO:
    fn visit_impt(&mut self, _: Symbol, _: Region) -> Option<Symbol> {
        None
    }

    fn visit_block(&mut self, block: &Block) -> Option<Symbol> {
        for stmt in block {
            walk_stmt(self, stmt);
        }
        None
    }
}

/// Only [`CodeGenerator::visit_expr`] returns a string (the name of temporary it generates)
impl ExprVisitor<Option<Symbol>> for CodeGenerator {
    fn visit_lit_expr(&mut self, lit: Lit, _: Region) -> Option<Symbol> {
        match lit {
            Lit::Int(k) => {
                let t = self.new_tmp_var();
                self.emit_assign_const_int(t, k);
                Some(t)
            }

            Lit::Str(str) => {
                let t = self.new_tmp_var();
                self.emit_assign_const_str(t, str);
                Some(t)
            }
            Lit::Char(k) => {
                let t = self.new_tmp_var();
                self.emit_assign_const_int(t, k as i64);
                Some(t)
            }
        }
    }

    fn visit_variable_expr(&mut self, var: Symbol, _: Region) -> Option<Symbol> {
        let t = self.new_tmp_var();
        self.emit_assign_var(t, var);
        Some(t)
    }

    fn visit_unary_expr(&mut self, _: UnOp, rhs: &Expr, _: Region) -> Option<Symbol> {
        todo!()
    }

    fn visit_binary_expr(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, _: Region) -> Option<Symbol> {
        let t1 = walk_expr(self, lhs).unwrap();
        let t2 = walk_expr(self, rhs).unwrap();
        let t = self.new_tmp_var();
        self.emit_assign_binary(t.clone(), op, t1, t2);
        Some(t)
    }

    fn visit_call_expr(&mut self, func_name: Symbol, args: &Vec<Box<Expr>>, _: Region) -> Option<Symbol> {
        for arg in args {
            let t = walk_expr(self, arg).unwrap();
            self.emit_arg(t);
        }
        let t = self.new_tmp_var();
        self.emit_assign_call(t.clone(), func_name.clone());

        Some(t)
    }

    fn visit_cond_expr(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        _: Region,
    ) -> Option<Symbol> {
        let t = walk_expr(self, condition).unwrap();
        let label = self.new_tmp_label();
        self.emit_ifz(t, label.clone());
        self.visit_block(then_block);
        self.emit_label(label);
        if let Some(ref else_block) = else_block {
            self.visit_block(else_block);
        }

        // TODO:
        None
    }

    fn visit_assign_expr(&mut self, var_name: Symbol, value: &Expr, _: Region) -> Option<Symbol> {
        let t = walk_expr(self, value).unwrap();
        self.emit_assign_var(var_name, t);
        Some(var_name)
    }
}
