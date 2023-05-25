//! `codegen`: Generates TAC IR from a Kip AST. Or in other words, this module generates an
//! intermediate representation of kip source code in three-address code. Three-address code is
//! high-level enough that it's machine indepedent, but low level enough that it's easy (enough) to
//! convert to assembly/machine code.

use std::collections::HashMap;

use crate::ast::visit::Visitor as AstVisitor;
use crate::ast::*;
use crate::interner::Symbol;

/// [`ic`]: intermediate code
/// TODO: remove this module
mod ic {
    use crate::interner::Symbol;
    use std::fmt;

    pub use crate::ast::BinOp;

    #[derive(Clone)]
    pub struct Label {
        pub name: Symbol,
    }

    impl Label {
        pub fn new(name: Symbol) -> Self {
            Self { name }
        }
        pub fn from_str(name: &str) -> Self {
            Self {
                name: Symbol::intern(name),
            }
        }
    }

    impl fmt::Display for Label {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    pub enum Instruction {
        // psuedo instruction
        Label(Symbol),
        Assign(Symbol, Expr),
        Goto(Label),
        /// [`Ifz`]: if-zero
        /// ifz _t0 goto _L0
        Ifz(Primary, Label),
        Arg(Primary),
        Ret(Option<Primary>),
    }

    impl Instruction {
        /// Returns `true` if the instruction is [`Ifz`].
        pub fn is_ifz(&self) -> bool {
            matches!(self, Self::Ifz(..))
        }

        /// Returns `true` if the instruction is [`Label`].
        pub fn is_label(&self) -> bool {
            matches!(self, Self::Label(..))
        }
    }

    impl fmt::Display for Instruction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Label(name) => write!(f, "{}:", name),
                Self::Assign(var, init) => write!(f, "{} := {}", var, init),
                Self::Goto(label) => write!(f, "goto {}", label),
                Self::Ifz(value, label) => write!(f, "ifz {} goto {}", value, label),
                Self::Arg(value) => write!(f, "arg {}", value),
                Self::Ret(value) => {
                    if let Some(value) = value {
                        write!(f, "ret {}", value)
                    } else {
                        write!(f, "ret")
                    }
                }
            }
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub enum Primary {
        Const(ConstKind),
        Var(Symbol),
    }

    impl fmt::Display for Primary {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Var(name) => name.fmt(f),
                Self::Const(kind) => kind.fmt(f),
            }
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub enum ConstKind {
        Int(i64),
        Str(Symbol),
    }

    impl fmt::Display for ConstKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Int(value) => value.fmt(f),
                Self::Str(string) => write!(f, "\"{}\"", string.as_str().escape_debug()),
            }
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    pub enum Expr {
        Call(Symbol),
        Binary(BinOp, Primary, Primary),
        Primary(Primary),
    }

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Binary(op, lhs, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
                Self::Primary(value) => write!(f, "{}", value),
                Self::Call(name) => write!(f, "call {}", name),
            }
        }
    }
}

fn blocks(instructions: &mut Vec<ic::Instruction>) -> Vec<&mut [ic::Instruction]> {
    instructions.split_inclusive_mut(|i| i.is_ifz()).collect()
}

fn elim_common_subexprs(block: &mut [ic::Instruction]) {
    use ic::Instruction::Assign;
    use ic::{Expr, Primary};

    let mut available_expressions: HashMap<Expr, Symbol> = HashMap::new();
    for instruction in block {
        if let Assign(var, expr) = instruction {
            if let Some(available_var) = available_expressions.get(expr) {
                *expr = Expr::Primary(Primary::Var(available_var.to_owned()));
            }
            available_expressions.insert(expr.clone(), var.to_owned());
        }
    }
}

#[derive(Default)]
pub struct CodeGenerator {
    instructions: Vec<ic::Instruction>,
    tmp_var: usize,
    tmp_label: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Default::default()
    }

    fn new_tmp_var(&mut self) -> Symbol {
        let t = format!("_t{}", self.tmp_var);
        self.tmp_var += 1;
        Symbol::intern(&t)
    }

    fn new_tmp_label(&mut self) -> ic::Label {
        use ic::Label;
        let name = format!("_L{}", self.tmp_label);
        self.tmp_label += 1;
        Label::from_str(&name)
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

    fn emit_ifz(&mut self, condition: Symbol, label: ic::Label) {
        self.instructions
            .push(ic::Instruction::Ifz(ic::Primary::Var(condition), label));
    }

    fn emit_label(&mut self, label: ic::Label) {
        self.instructions.push(ic::Instruction::Label(label.name));
    }

    fn emit_ret(&mut self, value: Option<Symbol>) {
        self.instructions
            .push(ic::Instruction::Ret(value.map(ic::Primary::Var)));
    }

    pub fn display_instructions(&mut self) {
        for instruction in &self.instructions {
            println!("{}", instruction);
        }

        self.optimize();

        println!("; optimized instructions");
        for instruction in &self.instructions {
            println!("{}", instruction);
        }
    }

    pub fn optimize(&mut self) {
        for block in blocks(&mut self.instructions) {
            elim_common_subexprs(block);
        }
    }
}

/// Only [`CodeGenerator::visit_expr`] returns a string (the name of temporary it generates)
impl AstVisitor<Option<Symbol>> for CodeGenerator {
    fn visit_expr(&mut self, e: &Expr) -> Option<Symbol> {
        match &e.kind {
            ExprKind::Lit(kind) => match kind {
                Lit::Int(k) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const_int(t, *k);
                    Some(t)
                }
                Lit::Str(str) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const_str(t.clone(), str.clone());
                    Some(t)
                }
                Lit::Char(k) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const_int(t.clone(), *k as i64);
                    Some(t)
                }
            },
            ExprKind::Variable(name) => {
                let t = self.new_tmp_var();
                self.emit_assign_var(t.clone(), name.clone());
                Some(t)
            }
            ExprKind::Binary(op, ref lhs, ref rhs) => {
                let t1 = self.visit_expr(lhs).unwrap();
                let t2 = self.visit_expr(rhs).unwrap();
                let t = self.new_tmp_var();
                self.emit_assign_binary(t.clone(), *op, t1, t2);
                Some(t)
            }
            ExprKind::Call(func, ref args) => {
                for arg in args {
                    let t = self.visit_expr(arg).unwrap();
                    self.emit_arg(t);
                }
                let t = self.new_tmp_var();
                self.emit_assign_call(t.clone(), func.clone());
                Some(t)
            }
            ExprKind::Cond(ref cond, ref then_branch, else_branch) => {
                let t = self.visit_expr(cond).unwrap();
                let label = self.new_tmp_label();
                self.emit_ifz(t, label.clone());
                self.visit_block(then_branch);
                self.emit_label(label);
                if let Some(ref else_block) = else_block {
                    self.visit_block(else_block);
                }

                // TODO:
                None
            }
            ExprKind::Assign(var, ref init) => {
                let t = self.visit_expr(init).unwrap();
                self.emit_assign_var(var.clone(), t);
                Some(var.clone())
            }
            ExprKind::Unary(_, _) => todo!(),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Option<Symbol> {
        match s.kind {
            StmtKind::Expr(ref expr) => {
                self.visit_expr(expr);
            }
            StmtKind::Var(name, ref init) => {
                let t = self.visit_expr(init).unwrap();
                self.emit_assign_var(name, t);
            }
            StmtKind::Ret(ref expr) => {
                let t = self.visit_expr(expr);
                self.emit_ret(t);
            }
        }

        None
    }

    fn visit_func(&mut self, f: &FuncDef) -> Option<Symbol> {
        use ic::Label;
        let label = Label::new(f.proto.name);
        self.emit_label(label);
        self.visit_block(&f.body)
    }

    fn visit_block(&mut self, b: &Block) -> Option<Symbol> {
        for s in &b.stmts {
            self.visit_stmt(s);
        }
        None
    }
}
