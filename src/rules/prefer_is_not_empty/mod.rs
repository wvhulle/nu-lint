use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{Fix, Replacement, context::LintContext, rule::Rule, violation::Violation};
fn check_subexpression_for_is_empty(block_id: nu_protocol::BlockId, context: &LintContext) -> bool {
    let block = context.working_set.get_block(block_id);
    let Some(pipeline) = block.pipelines.first() else {
        return false;
    };
    check_pipeline_for_is_empty(pipeline, context)
}
/// Check if an expression represents a "not ... is-empty" pattern
fn is_not_is_empty_pattern(expr: &Expression, context: &LintContext) -> bool {
    // Look for: not (expr | is-empty)
    let Expr::UnaryNot(inner_expr) = &expr.expr else {
        return false;
    };
    match &inner_expr.expr {
        Expr::Subexpression(block_id) => check_subexpression_for_is_empty(*block_id, context),
        Expr::FullCellPath(path) => {
            if let Expr::Subexpression(block_id) = &path.head.expr {
                check_subexpression_for_is_empty(*block_id, context)
            } else {
                false
            }
        }
        _ => false,
    }
}
fn check_pipeline_for_is_empty(pipeline: &Pipeline, context: &LintContext) -> bool {
    if pipeline.elements.len() >= 2 {
        // Check if the last element is "is-empty"
        if let Some(last_element) = pipeline.elements.last()
            && let Expr::Call(call) = &last_element.expr.expr
        {
            let decl = context.working_set.get_decl(call.decl_id);
            return decl.name() == "is-empty";
        }
    }
    false
}
fn extract_pipeline_text(pipeline: &Pipeline, context: &LintContext) -> Option<String> {
    if pipeline.elements.len() < 2 {
        return None;
    }
    let elements_before_is_empty = &pipeline.elements[..pipeline.elements.len() - 1];
    if elements_before_is_empty.is_empty() {
        return None;
    }
    let start_span = elements_before_is_empty.first().unwrap().expr.span;
    let end_span = elements_before_is_empty.last().unwrap().expr.span;
    let combined_span = nu_protocol::Span::new(start_span.start, end_span.end);
    let expr_text = &context.source[combined_span.start..combined_span.end];
    Some(format!("{} | is-not-empty", expr_text.trim()))
}
fn generate_fix_from_subexpression(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
) -> Option<String> {
    let block = context.working_set.get_block(block_id);
    let pipeline = block.pipelines.first()?;
    extract_pipeline_text(pipeline, context)
}
/// Generate the fix text for "not (expr | is-empty)" -> "expr | is-not-empty"
fn generate_fix_text(expr: &Expression, context: &LintContext) -> Option<String> {
    let Expr::UnaryNot(inner_expr) = &expr.expr else {
        return None;
    };
    match &inner_expr.expr {
        Expr::Subexpression(block_id) => generate_fix_from_subexpression(*block_id, context),
        Expr::FullCellPath(path) => {
            if let Expr::Subexpression(block_id) = &path.head.expr {
                generate_fix_from_subexpression(*block_id, context)
            } else {
                None
            }
        }
        _ => None,
    }
}
fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        // Check for "not ... is-empty" pattern
        if is_not_is_empty_pattern(expr, ctx)
            && let Some(fix_text) = generate_fix_text(expr, ctx)
        {
            let fix = Fix::new_static(
                "Replace 'not ... is-empty' with 'is-not-empty'",
                vec![Replacement::new_dynamic(expr.span, fix_text)],
            );
            vec![
                Violation::new_static(
                    "prefer_is_not_empty",
                    "Use 'is-not-empty' instead of 'not ... is-empty' for better readability",
                    expr.span,
                )
                .with_suggestion_static("Replace with 'is-not-empty'")
                .with_fix(fix),
            ]
        } else {
            vec![]
        }
    })
}
pub const fn rule() -> Rule {
    Rule::new(
        "prefer_is_not_empty",
        "Use 'is-not-empty' instead of 'not ... is-empty' for better readability",
        check,
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
