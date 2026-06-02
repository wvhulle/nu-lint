use nu_protocol::{
    VarId,
    ast::{Call, Comparison, Expr, Expression, MatchPattern, Operator, Pattern},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_flag_usage_in_body(call: &Call, context: &LintContext) -> Vec<(Detection, ())> {
    let Some(def) = call.custom_command_def(context) else {
        return vec![];
    };

    let body_block = context.working_set.get_block(def.body);
    let signature = &body_block.signature;

    signature
        .named
        .iter()
        .filter_map(|flag| {
            let var_id = flag.var_id?;

            // Boolean switches (flags without a type annotation) are never null.
            // They are `true` when present and `false` when absent.
            flag.arg.as_ref()?;

            if flag.default_value.is_some() {
                return None;
            }

            let var = context.working_set.get_variable(var_id);
            let flag_span = var.declaration_span;

            let null_checked_expr_spans = body_block.find_expr_spans(context, |expr, ctx| {
                has_null_comparison_for_var(expr, var_id, ctx)
            });

            let all_usage_spans = body_block.var_usages(var_id, context);

            let usage_span = all_usage_spans
                .iter()
                .find(|usage_span| {
                    !null_checked_expr_spans
                        .iter()
                        .any(|null_check_span| null_check_span.contains_span(**usage_span))
                })
                .copied()?;

            let flag_name = flag.short.map_or_else(
                || format!("--{}", flag.long),
                |short| format!("--{} (-{})", flag.long, short),
            );

            let detection = Detection::from_global_span(
                format!(
                    "Typed flag '{flag_name}' is used without checking if it is null. Typed flags \
                     (flags with `: type`) are optional and may be null when not provided"
                ),
                usage_span,
            )
            .with_primary_label("typed flag used without null check")
            .with_extra_label("flag declared here", flag_span);

            Some((detection, ()))
        })
        .collect()
}

fn is_null_pattern(pattern: &MatchPattern) -> bool {
    matches!(&pattern.pattern, Pattern::Expression(e) if matches!(&e.expr, Expr::Nothing))
}

fn matches_scrutinee(call: &Call, var_id: VarId) -> bool {
    call.get_first_positional_arg()
        .is_some_and(|s| s.matches_var(var_id))
}

fn has_null_first_arm(call: &Call) -> bool {
    matches!(
        call.get_positional_arg(1).map(|a| &a.expr),
        Some(Expr::MatchBlock(arms)) if arms.first().is_some_and(|(p, _)| is_null_pattern(p))
    )
}

fn is_match_with_null_arm_for_var(call: &Call, var_id: VarId, context: &LintContext) -> bool {
    call.get_call_name(context) == "match"
        && matches_scrutinee(call, var_id)
        && has_null_first_arm(call)
}

fn is_direct_null_equality(
    left: &Expression,
    op: &Expression,
    right: &Expression,
    var_id: VarId,
) -> bool {
    let is_eq_op = matches!(
        &op.expr,
        Expr::Operator(Operator::Comparison(
            Comparison::NotEqual | Comparison::Equal
        ))
    );
    let is_var_null_pair = (left.matches_var(var_id) && matches!(&right.expr, Expr::Nothing))
        || (right.matches_var(var_id) && matches!(&left.expr, Expr::Nothing));
    is_eq_op && is_var_null_pair
}

fn is_empty_pipeline_on_var(
    block_id: nu_protocol::BlockId,
    var_id: VarId,
    context: &LintContext,
) -> bool {
    context
        .working_set
        .get_block(block_id)
        .pipelines
        .iter()
        .any(|p| {
            p.elements.len() == 2
                && p.elements[0].expr.contains_variable(var_id)
                && matches!(
                    &p.elements[1].expr.expr,
                    Expr::Call(c) if matches!(c.get_call_name(context).as_str(), "is-empty" | "is-not-empty")
                )
        })
}

fn is_atomic_null_check(expr: &Expression, var_id: VarId, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::BinaryOp(left, op, right) => is_direct_null_equality(left, op, right, var_id),
        Expr::UnaryNot(inner) => inner.matches_var(var_id),
        Expr::Call(call) => is_match_with_null_arm_for_var(call, var_id, context),
        Expr::Subexpression(b) | Expr::Block(b) | Expr::Closure(b) => {
            is_empty_pipeline_on_var(*b, var_id, context)
        }
        _ => false,
    }
}

fn has_null_comparison_for_var(expr: &Expression, var_id: VarId, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;
    let mut found = Vec::new();
    expr.flat_map(
        context.working_set,
        &|e: &Expression| {
            if is_atomic_null_check(e, var_id, context) {
                vec![()]
            } else {
                vec![]
            }
        },
        &mut found,
    );
    !found.is_empty()
}

struct FlagCompareNull;

impl DetectFix for FlagCompareNull {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "check_typed_flag_before_use"
    }

    fn short_description(&self) -> &'static str {
        "Typed flag used without null check"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#flags")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                return check_flag_usage_in_body(call, ctx);
            }
            vec![]
        })
    }
}

pub static RULE: &dyn Rule = &FlagCompareNull;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
