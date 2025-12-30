use nu_protocol::{
    VarId,
    ast::{Call, Comparison, Expr, Expression, ListItem, Operator, RecordItem},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_flag_usage_in_body(call: &Call, context: &LintContext) -> Vec<(Detection, ())> {
    use nu_protocol::ast::Traverse;

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

            if flag.default_value.is_some() {
                return None;
            }

            let var = context.working_set.get_variable(var_id);
            let flag_span = var.declaration_span;

            let mut null_checks = Vec::new();
            body_block.flat_map(
                context.working_set,
                &|expr: &Expression| {
                    if has_null_comparison_for_var(expr, var_id) {
                        vec![()]
                    } else {
                        vec![]
                    }
                },
                &mut null_checks,
            );

            if !null_checks.is_empty() {
                return None;
            }

            let mut usage_spans = Vec::new();
            body_block.flat_map(
                context.working_set,
                &|expr: &Expression| {
                    if expr.matches_var(var_id) {
                        vec![expr.span]
                    } else {
                        vec![]
                    }
                },
                &mut usage_spans,
            );

            let usage_span = *usage_spans.first()?;

            let flag_name = flag.short.map_or_else(
                || format!("--{}", flag.long),
                |short| format!("--{} (-{})", flag.long, short),
            );

            let detection = Detection::from_global_span(
                format!(
                    "Flag '{flag_name}' is used without checking if it is null. Flags are always \
                     optional and may be null"
                ),
                usage_span,
            )
            .with_primary_label("flag used without null check")
            .with_extra_label("flag declared here", flag_span)
            .with_help(format!(
                "Check if the flag value is not null before using it: 'if ${} != null {{ ... }}'",
                flag.long
            ))
            .with_help(
                "Alternative: provide a default value in the function signature: '--{}: type = \
                 default-value'",
            );

            Some((detection, ()))
        })
        .collect()
}

fn has_null_comparison_for_var(expr: &Expression, var_id: VarId) -> bool {
    match &expr.expr {
        Expr::UnaryNot(inner) => {
            if inner.matches_var(var_id) {
                return true;
            }
            has_null_comparison_for_var(inner, var_id)
        }
        Expr::BinaryOp(left, op, right) => {
            let is_null_comparison = matches!(
                &op.expr,
                Expr::Operator(Operator::Comparison(
                    Comparison::NotEqual | Comparison::Equal
                ))
            );

            if is_null_comparison {
                let left_is_var = left.matches_var(var_id);
                let right_is_var = right.matches_var(var_id);
                let left_is_null = matches!(&left.expr, Expr::Nothing);
                let right_is_null = matches!(&right.expr, Expr::Nothing);

                if (left_is_var && right_is_null) || (left_is_null && right_is_var) {
                    return true;
                }
            }

            has_null_comparison_for_var(left, var_id) || has_null_comparison_for_var(right, var_id)
        }
        Expr::Call(call) => call
            .all_arg_expressions()
            .iter()
            .any(|arg| has_null_comparison_for_var(arg, var_id)),
        Expr::FullCellPath(path) => has_null_comparison_for_var(&path.head, var_id),
        Expr::List(items) => items.iter().any(|item| {
            let (ListItem::Item(e) | ListItem::Spread(_, e)) = item;
            has_null_comparison_for_var(e, var_id)
        }),
        Expr::StringInterpolation(items) => items
            .iter()
            .any(|item| has_null_comparison_for_var(item, var_id)),
        Expr::Table(table) => table
            .rows
            .iter()
            .flatten()
            .any(|cell| has_null_comparison_for_var(cell, var_id)),
        Expr::Record(fields) => fields.iter().any(|field| match field {
            RecordItem::Pair(key, val) => {
                has_null_comparison_for_var(key, var_id) || has_null_comparison_for_var(val, var_id)
            }
            RecordItem::Spread(_, e) => has_null_comparison_for_var(e, var_id),
        }),
        _ => false,
    }
}

struct FlagCompareNull;

impl DetectFix for FlagCompareNull {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "check_flag_before_use"
    }

    fn explanation(&self) -> &'static str {
        "Flags in custom commands are always optional and should be checked for null before use"
    }

    fn doc_url(&self) -> Option<&'static str> {
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
