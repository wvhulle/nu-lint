use nu_protocol::ast::{Expr, Expression, Pipeline};

use crate::{
    Fix, LintLevel, Replacement,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

enum InnerExprKind {
    Subexpression(nu_protocol::BlockId),
    FullCellPath { block_id: nu_protocol::BlockId },
}

struct IsNotEmptyFixData {
    span: nu_protocol::Span,
    inner: InnerExprKind,
}
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
    let expr_text = context.plain_text(combined_span);
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

fn check_not_is_empty(expr: &Expression, ctx: &LintContext) -> Vec<(Detection, IsNotEmptyFixData)> {
    if !is_not_is_empty_pattern(expr, ctx) {
        return vec![];
    }

    let Expr::UnaryNot(inner_expr) = &expr.expr else {
        return vec![];
    };

    let inner = match &inner_expr.expr {
        Expr::Subexpression(block_id) => InnerExprKind::Subexpression(*block_id),
        Expr::FullCellPath(path) => {
            if let Expr::Subexpression(block_id) = &path.head.expr {
                InnerExprKind::FullCellPath {
                    block_id: *block_id,
                }
            } else {
                return vec![];
            }
        }
        _ => return vec![],
    };

    let not_span = nu_protocol::Span::new(expr.span.start, expr.span.start + 3);

    let violation = Detection::from_global_span(
        "Use 'is-not-empty' instead of 'not ... is-empty' for better readability",
        not_span,
    )
    .with_primary_label("negation operator")
    .with_extra_label("is-empty check", inner_expr.span)
    .with_help("Replace with 'is-not-empty'");

    let fix_data = IsNotEmptyFixData {
        span: expr.span,
        inner,
    };

    vec![(violation, fix_data)]
}

struct UseBuiltinIsNotEmpty;

impl DetectFix for UseBuiltinIsNotEmpty {
    type FixInput<'a> = IsNotEmptyFixData;

    fn id(&self) -> &'static str {
        "use_is_not_empty"
    }

    fn explanation(&self) -> &'static str {
        "Use 'is-not-empty' instead of 'not ... is-empty' for better readability"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/is-not-empty.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(check_not_is_empty)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let block_id = match fix_data.inner {
            InnerExprKind::Subexpression(id) | InnerExprKind::FullCellPath { block_id: id } => id,
        };

        let fix_text = generate_fix_from_subexpression(block_id, context)?;

        Some(Fix::with_explanation(
            "Replace 'not ... is-empty' with 'is-not-empty'",
            vec![Replacement::new(fix_data.span, fix_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &UseBuiltinIsNotEmpty;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
