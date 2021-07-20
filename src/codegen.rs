//! Code Generation (codegen)

/* use crate::ast::{visit::Visitor, Expr, FuncDef, FuncProto, Stmt};

struct CodeGenerator {
    ir: String,
}

pub trait Codegen {
    fn codegen(&self, ir: &mut IRBuilder);
}

pub struct IRBuilder {
    /// kip intermediate representation
    ir: String,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self { ir: String::new() }
    }

    /// emits an instruction like:
    /// ```asm
    /// push 42
    /// ```
    pub fn emit_push_const(&mut self, v: i64) {
        self.ir.push_str(&format!("push {}\n", v));
    }

    /// emits an instruction like:
    /// ```asm
    /// push @v
    /// ```
    pub fn emit_push_local(&mut self, v: usize) {
        self.ir.push_str(&format!("push [rbp+{}]\n", v));
    }

    /// emits an instruction like:
    /// ```asm
    /// pop rax
    /// pop rcx
    /// add rax, rcx
    /// ```
    pub fn emit_push_add(&mut self) {
        self.ir.push_str(concat!(
            "pop rax\n",
            "pop rcx\n",
            "add rax, rcx\n",
            "push rax\n"
        ));
    }

    pub fn emit_mov(&mut self) {
        todo!()
    }
}

impl Codegen for Expr {
    fn codegen(&self, ir: &mut IRBuilder) {
        use Expr::*;

        match self {
            Binary { op, lhs, rhs } => {
                lhs.codegen(ir);
                rhs.codegen(ir);

                match *op {
                    '+' => todo!(),
                    _ => todo!(),
                }
            }
            IntLit(v) => ir.emit_push_const(*v),
            Var(_) => todo!(),
            Call { callee: _, args: _ } => todo!(),
        }
    }
}

impl Codegen for FuncProto {
    fn codegen(&self, ir: &mut IRBuilder) {
        todo!()
    }
}

impl Codegen for Stmt {
    fn codegen(&self, ir: &mut IRBuilder) {
        match self {
            Stmt::Expr(e) => e.codegen(ir),
            Stmt::VarDef { name: _, init: _ } => todo!(),
        }
    }
} */
