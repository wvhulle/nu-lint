use nu_protocol::ast::{Expr, Expression, Range};

use crate::{ast::call::CallExt, context::LintContext};

pub mod each_if_to_where;
pub mod for_filter_to_where;
pub mod omit_it_in_row_condition;
pub mod slice_to_drop;
pub mod slice_to_last;
pub mod slice_to_skip;
pub mod slice_to_take;
pub mod where_closure_to_it_condition;

/// Extracts an integer value from an expression, handling direct integers,
/// subexpressions, and full cell paths.
pub fn extract_int_value(expr: &Expression, context: &LintContext) -> Option<i64> {
    match &expr.expr {
        Expr::Int(n) => Some(*n),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Subexpression(block_id) | Expr::Block(block_id) => {
                let block = context.working_set.get_block(*block_id);
                block
                    .pipelines
                    .first()
                    .and_then(|pipeline| pipeline.elements.first())
                    .and_then(|elem| extract_int_value(&elem.expr, context))
            }
            _ => None,
        },
        Expr::Subexpression(block_id) | Expr::Block(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .pipelines
                .first()
                .and_then(|pipeline| pipeline.elements.first())
                .and_then(|elem| extract_int_value(&elem.expr, context))
        }
        _ => None,
    }
}

/// Gets the range argument from a slice call, if it exists and is valid.
pub fn get_slice_range<'a>(expr: &'a Expression, context: &LintContext) -> Option<&'a Range> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("slice", context) {
        return None;
    }

    let range_arg = call.get_first_positional_arg()?;

    match &range_arg.expr {
        Expr::Range(range) => Some(range),
        _ => None,
    }
}

/// Checks if an expression represents a negative value by examining the AST
/// structure. This handles constants like (-2), variables like (-$n), and
/// subexpressions like (-($x + 1)).
pub fn is_negative_expression(expr: &Expression, context: &LintContext) -> bool {
    match &expr.expr {
        // Direct negative integer
        Expr::Int(n) => *n < 0,
        // FullCellPath wrapping a subexpression - check inside
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Subexpression(block_id) | Expr::Block(block_id) => {
                let block = context.working_set.get_block(*block_id);
                block
                    .pipelines
                    .first()
                    .and_then(|pipeline| pipeline.elements.first())
                    .is_some_and(|elem| is_negative_expression(&elem.expr, context))
            }
            _ => false,
        },
        // Subexpression directly - check inside
        Expr::Subexpression(block_id) | Expr::Block(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .pipelines
                .first()
                .and_then(|pipeline| pipeline.elements.first())
                .is_some_and(|elem| is_negative_expression(&elem.expr, context))
        }
        // BinaryOp with subtraction where left is 0 (unary minus)
        Expr::BinaryOp(left, op, _right) => {
            use nu_protocol::ast::{Math, Operator};
            // Check if it's 0 - something or just a negation pattern
            matches!(op.expr, Expr::Operator(Operator::Math(Math::Subtract)))
                && extract_int_value(left, context) == Some(0)
        }
        _ => false,
    }
}
