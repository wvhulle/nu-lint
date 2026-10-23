use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

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
        .any(|elem| elem.expr.is_counter_increment(counter_name, context));

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

            (init_value.extract_int_value(context) == Some(0)).then_some((var_name, expr.span))
        })
        .flatten()
}

struct LoopCounter;

impl DetectFix for LoopCounter {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "loop_counter_to_range"
    }

    fn short_description(&self) -> &'static str {
        "Loop counter to range iteration"
    }

    fn source_link(&self) -> Option<&'static str> {
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
                        .with_extra_label("loop using counter with break", call.span()),
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
