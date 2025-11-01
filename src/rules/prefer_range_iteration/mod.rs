use nu_protocol::ast::{Assignment, Comparison, Expr, Expression, Math, Operator};

use crate::{
    ast::{CallExt, ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Extract an integer value from an expression, unwrapping blocks if needed
fn extract_int_value(expr: &Expression, context: &LintContext) -> Option<i64> {
    match &expr.expr {
        Expr::Int(n) => Some(*n),
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
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

/// Extract expression from block wrapper if present
fn unwrap_block_expr<'a>(expr: &'a Expression, context: &'a LintContext) -> &'a Expression {
    match &expr.expr {
        Expr::Block(block_id) | Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .pipelines
                .first()
                .and_then(|pipeline| pipeline.elements.first())
                .map_or(expr, |elem| &elem.expr)
        }
        _ => expr,
    }
}

/// Check if this is a counter comparison: `$counter < max`
fn is_counter_comparison(expr: &Expression, counter_name: &str, context: &LintContext) -> bool {
    matches!(
        &expr.expr,
        Expr::BinaryOp(left, op, _)
            if left.extract_variable_name(context).as_deref() == Some(counter_name)
                && matches!(
                    &op.expr,
                    Expr::Operator(Operator::Comparison(
                        Comparison::LessThan
                            | Comparison::LessThanOrEqual
                            | Comparison::GreaterThan
                            | Comparison::GreaterThanOrEqual
                    ))
                )
    )
}

/// Check if binary operation is `$counter + 1`
fn is_add_one_binop(
    left: &Expression,
    op: &Expression,
    right: &Expression,
    counter_name: &str,
    context: &LintContext,
) -> bool {
    left.extract_variable_name(context).as_deref() == Some(counter_name)
        && matches!(&op.expr, Expr::Operator(Operator::Math(Math::Add)))
        && matches!(&right.expr, Expr::Int(1))
}

/// Check if this is a counter increment: `$counter = $counter + 1` or `$counter
/// += 1`
fn is_counter_increment(expr: &Expression, counter_name: &str, context: &LintContext) -> bool {
    let Expr::BinaryOp(lhs, op, rhs) = &expr.expr else {
        return false;
    };

    let Expr::Operator(Operator::Assignment(assignment_op)) = &op.expr else {
        return false;
    };

    if lhs.extract_variable_name(context).as_deref() != Some(counter_name) {
        return false;
    }

    match assignment_op {
        Assignment::Assign => {
            let rhs_unwrapped = unwrap_block_expr(rhs, context);
            matches!(
                &rhs_unwrapped.expr,
                Expr::BinaryOp(add_left, add_op, add_right)
                    if is_add_one_binop(add_left, add_op, add_right, counter_name, context)
            )
        }
        Assignment::AddAssign => extract_int_value(rhs, context) == Some(1),
        _ => false,
    }
}

/// Check if block contains a counter increment
fn has_increment_in_block(
    block_id: nu_protocol::BlockId,
    counter_name: &str,
    context: &LintContext,
) -> bool {
    context
        .working_set
        .get_block(block_id)
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .any(|element| is_counter_increment(&element.expr, counter_name, context))
}

/// Extract counter declaration from `mut counter = 0`
fn extract_counter_from_mut(
    expr: &Expression,
    context: &LintContext,
) -> Option<(String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    (call.get_call_name(context) == "mut")
        .then(|| {
            let var_name = call.get_positional_arg(0)?.extract_variable_name(context)?;
            let init_value = call.get_positional_arg(1)?;

            (extract_int_value(init_value, context) == Some(0)).then_some((var_name, expr.span))
        })
        .flatten()
}

/// Check if while loop uses counter pattern and create violation
fn check_while_loop_for_counter(
    call: &nu_protocol::ast::Call,
    counter_name: &str,
    counter_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    let condition = call.get_positional_arg(0)?;
    let body_expr = call.get_positional_arg(1)?;
    let block_id = body_expr.extract_block_id()?;

    (is_counter_comparison(condition, counter_name, context)
        && has_increment_in_block(block_id, counter_name, context))
    .then(|| {
        RuleViolation::new_dynamic(
            "prefer_range_iteration",
            format!("While loop with counter '{counter_name}' - consider using range iteration"),
            counter_span,
        )
        .with_suggestion_static(
            "Use '1..$max | each { |i| ... }' instead of while loop with counter",
        )
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Find all `mut counter = 0` declarations
    let counters: Vec<(String, nu_protocol::Span)> = context
        .ast
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .filter_map(|element| extract_counter_from_mut(&element.expr, context))
        .collect();

    // Find while loops that use these counters
    counters
        .into_iter()
        .flat_map(|(counter_name, counter_span)| {
            context.collect_rule_violations(|expr, ctx| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };

                (call.get_call_name(ctx) == "while")
                    .then(|| check_while_loop_for_counter(call, &counter_name, counter_span, ctx))
                    .flatten()
                    .into_iter()
                    .collect()
            })
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_range_iteration",
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer range iteration over while loops with counters",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
