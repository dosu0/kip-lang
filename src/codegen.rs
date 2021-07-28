//! `codegen`: Generates TAC IR from a Kip AST. Or in other words, this module generates an
//! intermediate representation of kip source code in three-address code. Three-address code is
//! high-level enough that it's machine indepedent, but low level enough that it's easy (enough) to
//! convert to assembly/machine code.

use crate::ast::visit::Visitor as AstVisitor;
use crate::ast::*;

/// [`ic`]: intermediate code
mod ic {
    use std::fmt;

    pub use crate::ast::BinOp;

    #[derive(Clone)]
    pub struct Label {
        pub name: String,
    }

    impl Label {
        pub fn new<T: Into<String>>(name: T) -> Self {
            Label { name: name.into() }
        }
    }

    impl fmt::Display for Label {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    pub enum Instruction {
        // psuedo instruction
        Label(String),
        Assign(String, Expr),
        Goto(Label),
        /// [`Ifz`]: if-zero
        /// ifz _t0 goto _L0
        Ifz(Primary, Label),
        Arg(Primary),
        Ret(Option<Primary>),
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

    pub enum Primary {
        Const(i64),
        Var(String),
    }

    impl fmt::Display for Primary {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Var(name) => name.fmt(f),
                Self::Const(value) => value.fmt(f),
            }
        }
    }

    pub enum Expr {
        Call(String),
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

    fn new_tmp_var(&mut self) -> String {
        let t = format!("_t{}", self.tmp_var);
        self.tmp_var += 1;
        t
    }

    fn new_tmp_label(&mut self) -> ic::Label {
        use ic::Label;
        let name = format!("_L{}", self.tmp_var);
        self.tmp_label += 1;
        Label::new(name)
    }

    // wrapper types suck fr

    fn emit_assign_const(&mut self, name: String, init: i64) {
        self.emit_assign(name, ic::Expr::Primary(ic::Primary::Const(init)));
    }

    fn emit_assign_var(&mut self, name: String, init: String) {
        self.emit_assign(name, ic::Expr::Primary(ic::Primary::Var(init)));
    }

    fn emit_assign(&mut self, name: String, init: ic::Expr) {
        self.instructions.push(ic::Instruction::Assign(name, init));
    }

    fn emit_assign_call(&mut self, name: String, func: String) {
        self.instructions
            .push(ic::Instruction::Assign(name, ic::Expr::Call(func)));
    }

    fn emit_assign_binary(&mut self, name: String, op: ic::BinOp, lhs: String, rhs: String) {
        self.instructions.push(ic::Instruction::Assign(
            name,
            ic::Expr::Binary(op, ic::Primary::Var(lhs), ic::Primary::Var(rhs)),
        ));
    }

    fn emit_arg(&mut self, name: String) {
        self.instructions
            .push(ic::Instruction::Arg(ic::Primary::Var(name)));
    }

    fn emit_ifz(&mut self, condition: String, label: ic::Label) {
        self.instructions
            .push(ic::Instruction::Ifz(ic::Primary::Var(condition), label));
    }

    fn emit_label(&mut self, label: ic::Label) {
        self.instructions.push(ic::Instruction::Label(label.name));
    }

    fn emit_ret(&mut self, value: Option<String>) {
        self.instructions
            .push(ic::Instruction::Ret(value.map(|v| ic::Primary::Var(v))));
    }

    pub fn display_instructions(&self) {
        for instruction in &self.instructions {
            println!("{}", instruction);
        }
    }
}

/// Only [`CodeGenerator::visit_expr`] returns a string (the name of temporary it generates)
impl AstVisitor<Option<String>> for CodeGenerator {
    fn visit_expr(&mut self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Lit(kind) => match kind {
                LitKind::Int(k) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const(t.clone(), *k);
                    Some(t)
                }
                LitKind::Str(_) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const(t.clone(), 9000);
                    Some(t)
                }
                LitKind::Char(k) => {
                    let t = self.new_tmp_var();
                    self.emit_assign_const(t.clone(), *k as i64);
                    Some(t)
                }
            },
            ExprKind::Var(name) => {
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
            ExprKind::Cond(ref cond, ref if_block, else_block) => {
                let t = self.visit_expr(cond).unwrap();
                let label = self.new_tmp_label();
                self.emit_ifz(t, label.clone());
                self.visit_block(if_block);
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
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Option<String> {
        match &s.kind {
            StmtKind::Expr(ref expr) => {
                self.visit_expr(expr);
            }
            StmtKind::VarDef(name, ref init) => {
                let t = self.visit_expr(init).unwrap();
                self.emit_assign_var(name.clone(), t);
            }
            StmtKind::Ret(ref expr) => {
                let t = self.visit_expr(expr);
                self.emit_ret(t);
            }
        }

        None
    }

    fn visit_func(&mut self, f: &FuncDef) -> Option<String> {
        use ic::Label;
        let label = Label::new(&f.proto.name);
        self.emit_label(label);
        self.visit_block(&f.body)
    }

    fn visit_block(&mut self, b: &Block) -> Option<String> {
        for s in &b.stmts {
            self.visit_stmt(s);
        }
        None
    }
}
