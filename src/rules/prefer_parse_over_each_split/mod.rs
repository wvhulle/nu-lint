use nu_protocol::ast::Expr;

use crate::{
    ast::CallExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn contains_split_row(expr: &nu_protocol::ast::Expression, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let name = call.get_call_name(ctx);
            (name == "split row" || name == "split")
                || call.arguments.iter().any(|arg| match arg {
                    nu_protocol::ast::Argument::Positional(e)
                    | nu_protocol::ast::Argument::Named((_, _, Some(e))) => {
                        contains_split_row(e, ctx)
                    }
                    _ => false,
                })
        }
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => ctx
            .working_set
            .get_block(*id)
            .pipelines
            .iter()
            .flat_map(|pipeline| &pipeline.elements)
            .any(|element| contains_split_row(&element.expr, ctx)),
        Expr::FullCellPath(cell_path) => contains_split_row(&cell_path.head, ctx),
        Expr::BinaryOp(left, _, right) => {
            contains_split_row(left, ctx) || contains_split_row(right, ctx)
        }
        Expr::UnaryNot(inner) => contains_split_row(inner, ctx),
        _ => false,
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use crate::ast::CallExt;

    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) if call.is_call_to_command("each", ctx) => {
            let has_split = call
                .arguments
                .iter()
                .filter_map(|arg| match arg {
                    nu_protocol::ast::Argument::Positional(expr) => Some(expr),
                    _ => None,
                })
                .any(|expr| contains_split_row(expr, ctx));

            if has_split {
                vec![
                    RuleViolation::new_static(
                        "prefer_parse_over_each_split",
                        "Manual splitting with 'each' and 'split row' - consider using 'parse'",
                        call.span(),
                    )
                    .with_suggestion_static(
                        "Use 'parse \"{field1} {field2}\"' for structured text extraction instead \
                         of 'each' with 'split row'",
                    ),
                ]
            } else {
                vec![]
            }
        }
        _ => vec![],
    })
}

pub(crate) fn rule() -> Rule {
    Rule::new(
        "prefer_parse_over_each_split",
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer 'parse' over 'each' with 'split row' for structured text processing",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
