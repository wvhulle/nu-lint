use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn block_contains_split_row(block_id: nu_protocol::BlockId, ctx: &LintContext) -> bool {
    ctx.working_set
        .get_block(block_id)
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.elements)
        .any(|element| expr_contains_split_row(&element.expr, ctx))
}

fn expr_contains_split_row(expr: &nu_protocol::ast::Expression, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            let name = ctx.working_set.get_decl(call.decl_id).name();
            (name == "split row" || name == "split")
                || call.arguments.iter().any(|arg| match arg {
                    nu_protocol::ast::Argument::Positional(e)
                    | nu_protocol::ast::Argument::Named((_, _, Some(e))) => {
                        expr_contains_split_row(e, ctx)
                    }
                    _ => false,
                })
        }
        Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => {
            block_contains_split_row(*id, ctx)
        }
        Expr::FullCellPath(cell_path) => expr_contains_split_row(&cell_path.head, ctx),
        Expr::BinaryOp(left, _, right) => {
            expr_contains_split_row(left, ctx) || expr_contains_split_row(right, ctx)
        }
        Expr::UnaryNot(inner) => expr_contains_split_row(inner, ctx),
        _ => false,
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => {
            let decl = ctx.working_set.get_decl(call.decl_id);
            if decl.name() == "each" {
                let has_split = call
                    .arguments
                    .iter()
                    .filter_map(|arg| match arg {
                        nu_protocol::ast::Argument::Positional(expr) => Some(expr),
                        _ => None,
                    })
                    .any(|expr| match &expr.expr {
                        Expr::Closure(id) | Expr::Block(id) => block_contains_split_row(*id, ctx),
                        _ => false,
                    });

                if has_split {
                    return vec![Violation {
                        rule_id: "prefer_parse_over_each_split".to_string().into(),
                        severity: Severity::Info,
                        message: "Manual splitting with 'each' and 'split row' - consider using \
                                  'parse'"
                            .to_string()
                            .into(),
                        span: call.span(),
                        suggestion: Some(
                            "Use 'parse \"{field1} {field2}\"' for structured text extraction \
                             instead of 'each' with 'split row'"
                                .to_string()
                                .into(),
                        ),
                        fix: None,
                        file: None,
                    }];
                }
            }
            vec![]
        }
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_parse_over_each_split",
        RuleCategory::Idioms,
        Severity::Info,
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
