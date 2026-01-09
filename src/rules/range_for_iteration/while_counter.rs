use nu_protocol::ast::{Assignment, Call, Comparison, Expr, Expression, Math, Operator};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

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

fn check_while_loop_for_counter(
    call: &Call,
    counter_name: &str,
    counter_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<Detection> {
    let condition = call.get_positional_arg(0)?;
    let body_expr = call.get_positional_arg(1)?;
    let block_id = body_expr.extract_block_id()?;

    (is_counter_comparison(condition, counter_name, context)
        && has_increment_in_block(block_id, counter_name, context))
    .then(|| {
        Detection::from_global_span(
            format!(
                "While loop with counter '{counter_name}' can be replaced with range iteration"
            ),
            counter_span,
        )
        .with_primary_label("counter initialization")
        .with_extra_label("while loop using counter", call.span())
    })
}

struct WhileCounter;

impl DetectFix for WhileCounter {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "replace_counter_while_with_each"
    }

    fn short_description(&self) -> &'static str {
        "Replace while loop over a numerical counter with range iteration and functional style \
         pipe into each."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/each.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let violations = context
            .ast
            .pipelines
            .iter()
            .flat_map(|pipeline| &pipeline.elements)
            .filter_map(|element| extract_counter_from_mut(&element.expr, context))
            .flat_map(|(counter_name, counter_span)| {
                context.detect(|expr, ctx| {
                    let Expr::Call(call) = &expr.expr else {
                        return vec![];
                    };

                    (call.get_call_name(ctx) == "while")
                        .then(|| {
                            check_while_loop_for_counter(call, &counter_name, counter_span, ctx)
                        })
                        .flatten()
                        .into_iter()
                        .collect()
                })
            })
            .collect();
        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &WhileCounter;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
