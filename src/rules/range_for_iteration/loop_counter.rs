use nu_protocol::ast::{Assignment, Expr, Expression, Math, Operator};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
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

/// Check if an if statement has a break condition comparing counter to a limit
fn has_break_with_counter_check(
    expr: &Expression,
    counter_name: &str,
    context: &LintContext,
) -> bool {
    let Expr::Call(call) = &expr.expr else {
        return false;
    };

    if call.get_call_name(context) != "if" {
        return false;
    }

    let Some(condition) = call.get_positional_arg(0) else {
        return false;
    };

    let Some(then_block_expr) = call.get_positional_arg(1) else {
        return false;
    };

    let Some(then_block_id) = then_block_expr.extract_block_id() else {
        return false;
    };

    let then_block = context.working_set.get_block(then_block_id);

    let has_counter_in_condition = condition.extract_variable_name(context).as_deref()
        == Some(counter_name)
        || matches!(&condition.expr, Expr::BinaryOp(left, _, _)
            if left.extract_variable_name(context).as_deref() == Some(counter_name));

    let has_break = then_block.all_elements().iter().any(|elem| {
        matches!(&elem.expr.expr, Expr::Call(call) if call.get_call_name(context) == "break")
    });

    has_counter_in_condition && has_break
}

fn has_counter_pattern_in_loop_block(
    block_id: nu_protocol::BlockId,
    counter_name: &str,
    context: &LintContext,
) -> bool {
    let block = context.working_set.get_block(block_id);
    let all_elements = block.all_elements();
    let elements: Vec<_> = all_elements.iter().collect();

    let has_break_check = elements
        .iter()
        .any(|elem| has_break_with_counter_check(&elem.expr, counter_name, context));

    let has_increment = elements
        .iter()
        .any(|elem| is_counter_increment(&elem.expr, counter_name, context));

    has_break_check && has_increment
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

struct LoopCounter;

impl DetectFix for LoopCounter {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "replace_loop_counter_with_range"
    }

    fn explanation(&self) -> &'static str {
        "Replace infinite loop with counter and break with range iteration"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/each.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let detections = context
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

                    if call.get_call_name(ctx) != "loop" {
                        return vec![];
                    }

                    let Some(body_expr) = call.get_positional_arg(0) else {
                        return vec![];
                    };

                    let Some(block_id) = body_expr.extract_block_id() else {
                        return vec![];
                    };

                    if !has_counter_pattern_in_loop_block(block_id, &counter_name, ctx) {
                        return vec![];
                    }

                    vec![
                        Detection::from_global_span(
                            format!(
                                "Infinite loop with counter '{counter_name}' can be replaced with \
                                 range iteration"
                            ),
                            counter_span,
                        )
                        .with_primary_label("counter initialization")
                        .with_extra_label("loop using counter with break", call.span())
                        .with_help(
                            "Use '0..$max | each { |i| ... }' instead of loop with counter and \
                             break",
                        ),
                    ]
                })
            })
            .collect();
        Self::no_fix(detections)
    }
}

pub static RULE: &dyn Rule = &LoopCounter;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
