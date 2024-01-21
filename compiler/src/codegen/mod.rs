//! `codegen`: Generates TAC IR from a Kip AST. Or in other words, this module generates an
//! intermediate representation of kip source code in three-address code. Three-address code is
//! high-level enough that it's machine indepedent, but low level enough that it's easy (enough) to
//! convert to assembly/machine code.

/// use std::collections::HashMap;
use crate::ast::Lit;

use crate::ast::stmt::{Block, FuncProto, Module, Stmt};
use crate::ast::visit::*;
use crate::ast::{BinOp, Expr, Region, UnOp};
use crate::name::name;
use crate::name::Name as Symbol;

pub mod ic;
mod optimize;
mod visit;

use ic::Instruction;
use optimize::elim_common_subexprs;

use std::fmt::Write;

#[derive(Default)]
pub struct CodeGenerator {
    instructions: Vec<Instruction>,
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
            walk_stmt(self, stmt);
        }

        self.optimize_mut()
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
        self.instructions.push(Instruction::Assign(name, init));
    }

    fn emit_assign_call(&mut self, name: Symbol, func: Symbol) {
        self.instructions
            .push(Instruction::Assign(name, ic::Expr::Call(func)));
    }

    fn emit_assign_binary(&mut self, name: Symbol, op: ic::BinOp, lhs: Symbol, rhs: Symbol) {
        self.instructions.push(Instruction::Assign(
            name,
            ic::Expr::Binary(op, ic::Primary::Var(lhs), ic::Primary::Var(rhs)),
        ));
    }
    fn emit_arg(&mut self, name: Symbol) {
        self.instructions
            .push(Instruction::Arg(ic::Primary::Var(name)));
    }

    fn emit_ifz(&mut self, condition: Symbol, label: Symbol) {
        self.instructions
            .push(Instruction::Ifz(ic::Primary::Var(condition), label));
    }

    fn emit_label(&mut self, label: Symbol) {
        self.instructions.push(Instruction::Label(label));
    }

    fn emit_ret(&mut self, value: Option<Symbol>) {
        self.instructions
            .push(Instruction::Ret(value.map(ic::Primary::Var)));
    }

    fn emit_goto(&mut self, label: Symbol) {
        self.instructions.push(Instruction::Goto(label))
    }

    // FIXME: i should probably do this as a display trait or something
    pub fn intermediate_code(&self) -> String {
        let mut ic = String::new();
        // this tabs all instructions that arent labels
        for instruction in &self.instructions {
            match instruction {
                Instruction::Label(_) => writeln!(&mut ic, "{instruction}").unwrap(),
                _ => writeln!(&mut ic, "    {instruction}").unwrap(),
            }
        }

        ic
    }

    // FIXME: i should probably do this as a display trait or something
    pub fn optimized_intermediate_code(&self) -> String {
        let mut ic = String::new();
        let instructions = self.optimize();
        // this tabs all instructions that arent labels
        for instruction in instructions {
            match instruction {
                Instruction::Label(_) => writeln!(&mut ic, "{instruction}").unwrap(),
                _ => writeln!(&mut ic, "    {instruction}").unwrap(),
            }
        }

        ic
    }
    pub fn optimize_mut(&mut self) {
        // a block is a sequence of instructions where control enters and leaves the sequence at only
        // one place respectively
        let blocks = self
            .instructions
            .split_inclusive_mut(|i| i.is_ifz() || i.is_label());

        for basic_block in blocks {
            elim_common_subexprs(basic_block);
        }
    }

    fn optimize(&self) -> Vec<Instruction> {
        let mut new_instructions: Vec<Instruction> = Vec::with_capacity(self.instructions.len());
        let blocks = self
            .instructions
            .split_inclusive(|instr| instr.is_ifz() || instr.is_label());

        for block in blocks {
            new_instructions.extend(elim_common_subexprs(block));
        }

        new_instructions
    }
}
/// Only [`CodeGenerator::visit_expr`] returns a string (the name of temporary it generates)
impl StmtVisitor<Option<Symbol>> for CodeGenerator {
    fn visit_expr_stmt(&mut self, expr: &Expr, _: Region) -> Option<Symbol> {
        walk_expr(self, expr)
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

    fn visit_func(&mut self, proto: &FuncProto, body: &[Box<Stmt>], _: Region) -> Option<Symbol> {
        self.emit_label(proto.name);
        self.visit_block(body)
    }

    // TODO: implement something
    fn visit_extern(&mut self, _: &FuncProto, _: Region) -> Option<Symbol> {
        None
    }

    // TODO:
    fn visit_impt(&mut self, _: Symbol, _: Region) -> Option<Symbol> {
        None
    }

    fn visit_block(&mut self, block: &[Box<Stmt>]) -> Option<Symbol> {
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

    fn visit_unary_expr(&mut self, _: UnOp, _: &Expr, _: Region) -> Option<Symbol> {
        todo!()
    }

    fn visit_binary_expr(
        &mut self,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
        _: Region,
    ) -> Option<Symbol> {
        let t1 = walk_expr(self, lhs).unwrap();
        let t2 = walk_expr(self, rhs).unwrap();
        let t = self.new_tmp_var();
        self.emit_assign_binary(t, op, t1, t2);
        Some(t)
    }

    fn visit_call_expr(
        &mut self,
        func_name: Symbol,
        args: &[Box<Expr>],
        _: Region,
    ) -> Option<Symbol> {
        for arg in args {
            let t = walk_expr(self, arg).unwrap();
            self.emit_arg(t);
        }
        let t = self.new_tmp_var();
        self.emit_assign_call(t, func_name);

        Some(t)
    }

    fn visit_cond_expr(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        else_block: Option<&Block>,
        _: Region,
    ) -> Option<Symbol> {
        // evalute the condition into a temporary
        let t = walk_expr(self, condition).unwrap();

        // this label is
        let else_label = self.new_tmp_label();
        let end_label = self.new_tmp_label();

        self.emit_ifz(t, else_label);
        self.visit_block(then_block);
        if else_block.is_some() {
            self.emit_goto(end_label);
        }
        self.emit_label(else_label);
        if let Some(else_block) = else_block {
            self.visit_block(else_block);
            self.emit_label(end_label);
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
