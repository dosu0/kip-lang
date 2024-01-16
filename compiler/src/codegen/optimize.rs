/// Includes intermediate code optimization functions
///
use super::ic::{Expr, Instruction, Primary};
use crate::name::Name as Symbol;
use std::collections::HashMap;

pub fn elim_common_subexprs(block: &mut [Instruction]) {
    // this stores all the local expressions in a block
    let mut available_expressions: HashMap<&Expr, Symbol> = HashMap::new();

    // loops thru all the assignments in the block and adds them to the avaible expressions
    for instruction in block {
        if let Instruction::Assign(sym, expr) = instruction {
            // if the assign
            if let Some((_, replacement)) = available_expressions.get_key_value(expr) {
                *expr = Expr::Primary(Primary::Var(*replacement));
            }

            dbg!(&available_expressions);
            available_expressions.insert(expr, *sym);
        }
    }
}

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
}
