use nu_lint_analysis::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    VarId,
    ast::{Call, Comparison, Expr, Expression, Operator},
};

fn is_counter_comparison(expr: &Expression, counter_var_id: VarId) -> bool {
    matches!(
        &expr.expr,
        Expr::BinaryOp(left, op, _)
            if left.root_var_id() == Some(counter_var_id)
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

fn has_increment_in_block(
    block_id: nu_protocol::BlockId,
    counter_var_id: VarId,
    context: &LintContext,
) -> bool {
    context
        .working_set
        .get_block(block_id)
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .any(|element| element.expr.is_counter_increment(counter_var_id, context))
}

fn extract_counter_from_mut(
    expr: &Expression,
    context: &LintContext,
) -> Option<(VarId, String, nu_protocol::Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    (call.builtin_name(context) == Some("mut"))
        .then(|| {
            let name_arg = call.get_positional_arg(0)?;
            let counter_var_id = name_arg.root_var_id()?;
            let var_name = name_arg.extract_variable_name(context)?;
            let init_value = call.get_positional_arg(1)?;

            (init_value.extract_int_value(context) == Some(0)).then_some((
                counter_var_id,
                var_name,
                expr.span,
            ))
        })
        .flatten()
}

fn check_while_loop_for_counter(
    call: &Call,
    counter_var_id: VarId,
    counter_name: &str,
    counter_span: nu_protocol::Span,
    context: &LintContext,
) -> Option<Detection> {
    let condition = call.get_positional_arg(0)?;
    let body_expr = call.get_positional_arg(1)?;
    let block_id = body_expr.extract_block_id()?;

    (is_counter_comparison(condition, counter_var_id)
        && has_increment_in_block(block_id, counter_var_id, context))
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
        "while_counter_to_range"
    }

    fn short_description(&self) -> &'static str {
        "Counter while-loop to range iteration"
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
            .flat_map(|(counter_var_id, counter_name, counter_span)| {
                context.detect(|expr, ctx| {
                    let Expr::Call(call) = &expr.expr else {
                        return vec![];
                    };

                    (call.builtin_name(ctx) == Some("while"))
                        .then(|| {
                            check_while_loop_for_counter(
                                call,
                                counter_var_id,
                                &counter_name,
                                counter_span,
                                ctx,
                            )
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
