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
            if let Some((_, replacement)) = available_expressions.get_key_value(expr) {
                    *expr = Expr::Primary(Primary::Var(*replacement));
            }
            available_expressions.insert(expr, *sym);
        }
    }
}
