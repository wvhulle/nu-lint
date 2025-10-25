use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
};

/// Check if an expression represents a "not ... is-empty" pattern
fn is_not_is_empty_pattern(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    // Look for: not (expr | is-empty)
    if let Expr::UnaryNot(inner_expr) = &expr.expr {
        match &inner_expr.expr {
            Expr::Subexpression(block_id) => {
                let block = context.working_set.get_block(*block_id);
                if let Some(pipeline) = block.pipelines.first() {
                    return check_pipeline_for_is_empty(pipeline, context);
                }
            }
            Expr::FullCellPath(path) => {
                // Check if the head is a subexpression
                if let Expr::Subexpression(block_id) = &path.head.expr {
                    let block = context.working_set.get_block(*block_id);
                    if let Some(pipeline) = block.pipelines.first() {
                        return check_pipeline_for_is_empty(pipeline, context);
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn check_pipeline_for_is_empty(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> bool {
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

/// Generate the fix text for "not (expr | is-empty)" -> "expr | is-not-empty"
fn generate_fix_text(expr: &nu_protocol::ast::Expression, context: &LintContext) -> Option<String> {
    // Extract the expression before "is-empty" from "not (expr | is-empty)"
    if let Expr::UnaryNot(inner_expr) = &expr.expr {
        match &inner_expr.expr {
            Expr::Subexpression(block_id) => {
                let block = context.working_set.get_block(*block_id);
                if let Some(pipeline) = block.pipelines.first()
                    && pipeline.elements.len() >= 2
                {
                    // Get all elements except the last one (which is "is-empty")
                    let elements_before_is_empty =
                        &pipeline.elements[..pipeline.elements.len() - 1];
                    if !elements_before_is_empty.is_empty() {
                        let start_span = elements_before_is_empty.first().unwrap().expr.span;
                        let end_span = elements_before_is_empty.last().unwrap().expr.span;
                        let combined_span = nu_protocol::Span::new(start_span.start, end_span.end);
                        let expr_text = &context.source[combined_span.start..combined_span.end];
                        return Some(format!("{} | is-not-empty", expr_text.trim()));
                    }
                }
            }
            Expr::FullCellPath(path) => {
                if let Expr::Subexpression(block_id) = &path.head.expr {
                    let block = context.working_set.get_block(*block_id);
                    if let Some(pipeline) = block.pipelines.first()
                        && pipeline.elements.len() >= 2
                    {
                        // Get all elements except the last one (which is "is-empty")
                        let elements_before_is_empty =
                            &pipeline.elements[..pipeline.elements.len() - 1];
                        if !elements_before_is_empty.is_empty() {
                            let start_span = elements_before_is_empty.first().unwrap().expr.span;
                            let end_span = elements_before_is_empty.last().unwrap().expr.span;
                            let combined_span =
                                nu_protocol::Span::new(start_span.start, end_span.end);
                            let expr_text = &context.source[combined_span.start..combined_span.end];
                            return Some(format!("{} | is-not-empty", expr_text.trim()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| {
        // Check for "not ... is-empty" pattern
        if is_not_is_empty_pattern(expr, ctx)
            && let Some(fix_text) = generate_fix_text(expr, ctx)
        {
            let fix = Some(Fix {
                description: "Replace 'not ... is-empty' with 'is-not-empty'"
                    .to_string()
                    .into(),
                replacements: vec![Replacement {
                    span: expr.span,
                    new_text: fix_text.into(),
                }],
            });

            vec![Violation {
                rule_id: "prefer_is_not_empty".into(),
                severity: Severity::Info,
                message: "Use 'is-not-empty' instead of 'not ... is-empty' for better readability"
                    .to_string()
                    .into(),
                span: expr.span,
                suggestion: Some("Replace with 'is-not-empty'".to_string().into()),
                fix,
                file: None,
            }]
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_is_not_empty",
        RuleCategory::Idioms,
        Severity::Info,
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
