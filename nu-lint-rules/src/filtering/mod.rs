use nu_lint_analysis::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
};
use nu_protocol::ast::{Call, Comparison, Expr, Expression, Operator, Range};

pub mod each_if_to_where;
pub mod for_filter_to_where;
pub mod omit_it_in_row_condition;
pub mod slice_to_drop;
pub mod slice_to_last;
pub mod slice_to_skip;
pub mod slice_to_take;
pub mod uniq_to_set_operation;
pub mod where_closure_drop_parameter;
pub mod where_not_empty_to_compact;
pub mod where_not_null_to_compact;

#[must_use]
pub fn where_it_not_equal_matches(
    where_call: &Call,
    context: &LintContext,
    matches_compared_operand: impl Fn(&Expression) -> bool,
) -> bool {
    if where_call.builtin_name(context) != Some("where") {
        return false;
    }
    let Some(Expr::RowCondition(block_id)) = where_call
        .get_first_positional_arg()
        .map(|condition_arg| &condition_arg.expr)
    else {
        return false;
    };

    let block = context.working_set.get_block(*block_id);
    let Some(row_variable_id) = block
        .signature
        .required_positional
        .first()
        .and_then(|row_parameter| row_parameter.var_id)
    else {
        return false;
    };
    let [pipeline] = block.pipelines.as_slice() else {
        return false;
    };
    let [element] = pipeline.elements.as_slice() else {
        return false;
    };
    let Expr::BinaryOp(left_operand, operator, right_operand) = &element.expr.expr else {
        return false;
    };
    if !matches!(
        operator.expr,
        Expr::Operator(Operator::Comparison(Comparison::NotEqual))
    ) {
        return false;
    }

    (left_operand.extract_direct_var() == Some(row_variable_id)
        && matches_compared_operand(right_operand))
        || (right_operand.extract_direct_var() == Some(row_variable_id)
            && matches_compared_operand(left_operand))
}

/// Extracts an integer value from an expression, handling direct integers,
/// subexpressions, and full cell paths.
#[must_use]
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
#[must_use]
pub fn get_slice_range<'a>(expr: &'a Expression, context: &LintContext) -> Option<&'a Range> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if call.builtin_name(context) != Some("slice") {
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
#[must_use]
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
