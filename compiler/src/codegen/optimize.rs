/// Includes intermediate code optimization functions
///
use super::ic::{Expr, Instruction, Primary};
use crate::name::Name as Symbol;
use std::collections::HashMap;

pub fn elim_common_subexprs(block: &[Instruction]) -> Vec<Instruction> {
    let mut available_exprs: HashMap<Symbol, Expr> = HashMap::new();
    let mut new_instructions = Vec::with_capacity(block.len());

    for instruction in block {
        let mut optimized_instruction = *instruction;

        if let Instruction::Assign(symbol, expr) = instruction {
            for (available_symbol, available_expr) in available_exprs.iter() {
                if expr == available_expr {
                    optimized_instruction = Instruction::Assign(
                        *symbol,
                        Expr::Primary(Primary::Var(*available_symbol)),
                    );
                }
            }
            available_exprs.insert(*symbol, *expr);
        }

        new_instructions.push(optimized_instruction);
    }

    new_instructions
}

// TODO: implement copy propogation
/*
pub fn copy_propagation(block: &mut [Instruction]) {
    // this stores all the local expressions in a block
    let mut available_expressions: HashMap<Symbol, &Expr> = HashMap::new();

    // loops thru all the assignments in the block and adds them to the avaible expressions
    /* for instruction in block {
    if let Instruction::Assign(sym, expr) = instruction {

        match expr {
            Expr::Call(var2) => {
                if let Some((_, expr)) = available_expressions.get_key_value(var2) {
                }
            },
            Expr::Binary(var1, _, var2) => todo!(),
            Expr::Primary(var) => {
                if let Some((_, expr)) = available_expressions.get_key_value(var) {
                }
            },
        }

        available_expressions.insert(sym, expr);
    } */
} */
