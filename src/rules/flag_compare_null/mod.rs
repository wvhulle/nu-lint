use nu_protocol::{
    VarId,
    ast::{Call, Comparison, Expr, Expression, ListItem, Operator, RecordItem},
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

            let all_usage_spans =
                body_block.find_var_usage_spans(var_id, context, |_expr, _var_id, _ctx| true);

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

fn has_null_comparison_for_var(expr: &Expression, var_id: VarId, context: &LintContext) -> bool {
    match &expr.expr {
        Expr::UnaryNot(inner) => {
            if inner.matches_var(var_id) {
                return true;
            }
            has_null_comparison_for_var(inner, var_id, context)
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

            has_null_comparison_for_var(left, var_id, context)
                || has_null_comparison_for_var(right, var_id, context)
        }
        Expr::Call(call) => call
            .all_arg_expressions()
            .iter()
            .any(|arg| has_null_comparison_for_var(arg, var_id, context)),
        Expr::FullCellPath(path) => has_null_comparison_for_var(&path.head, var_id, context),
        Expr::List(items) => items.iter().any(|item| {
            let (ListItem::Item(e) | ListItem::Spread(_, e)) = item;
            has_null_comparison_for_var(e, var_id, context)
        }),
        Expr::StringInterpolation(items) => items
            .iter()
            .any(|item| has_null_comparison_for_var(item, var_id, context)),
        Expr::Table(table) => table
            .rows
            .iter()
            .flatten()
            .any(|cell| has_null_comparison_for_var(cell, var_id, context)),
        Expr::Record(fields) => fields.iter().any(|field| match field {
            RecordItem::Pair(key, val) => {
                has_null_comparison_for_var(key, var_id, context)
                    || has_null_comparison_for_var(val, var_id, context)
            }
            RecordItem::Spread(_, e) => has_null_comparison_for_var(e, var_id, context),
        }),
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            use nu_protocol::ast::Traverse;
            let block = context.working_set.get_block(*block_id);
            let mut found = Vec::new();
            block.flat_map(
                context.working_set,
                &|e: &Expression| {
                    if has_null_comparison_for_var(e, var_id, context) {
                        vec![()]
                    } else {
                        vec![]
                    }
                },
                &mut found,
            );
            !found.is_empty()
        }
        _ => false,
    }
}

struct FlagCompareNull;

impl DetectFix for FlagCompareNull {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "check_typed_flag_before_use"
    }

    fn short_description(&self) -> &'static str {
        "Typed flags (--flag: type) in custom commands are optional and should be checked for null \
         before use. Boolean switches (--flag without type) are always true/false and don't need \
         null checks."
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
