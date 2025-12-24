use nu_protocol::ast::{Expr, Expression, RecordItem, Traverse};

use crate::{
    LintLevel, ast::expression::ExpressionExt, context::LintContext, rule::Rule,
    violation::Violation,
};

const MAX_RECORD_LINE_LENGTH: usize = 80;

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Record(fields) = &expr.expr {
                if should_be_multiline(expr, fields, context) {
                    vec![create_violation(expr.span)]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        },
        &mut violations,
    );

    violations
}

fn should_be_multiline(expr: &Expression, fields: &[RecordItem], context: &LintContext) -> bool {
    let text = expr.span_text(context);

    if text.contains('\n') {
        return false;
    }

    // Should be multiline if:
    // 1. Single line AND more than 80 characters, OR
    // 2. Single line AND contains nested lists or records
    text.len() > MAX_RECORD_LINE_LENGTH || has_nested_structures(fields)
}

fn has_nested_structures(fields: &[RecordItem]) -> bool {
    fields.iter().any(|item| {
        let expr = match item {
            RecordItem::Pair(_, val) => val,
            RecordItem::Spread(_, expr) => expr,
        };

        // Handle FullCellPath wrapping
        let inner_expr = match &expr.expr {
            Expr::FullCellPath(full_cell_path) => &full_cell_path.head.expr,
            other => other,
        };

        matches!(inner_expr, Expr::List(_) | Expr::Record(_))
    })
}

fn create_violation(span: nu_protocol::Span) -> Violation {
    Violation::new(
        "Long records should use multiline format with each field on a separate line",
        span,
    )
    .with_help("Put each record field on a separate line for better readability")
}

/// This rule uses AST-based detection and is compatible with topiary-nushell
/// tree-sitter formatting. It analyzes actual record structures rather than
/// regex patterns.
pub const RULE: Rule = Rule::new(
    "wrap_wide_records",
    "Prefer multiline format for long or complex records",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/style_guide.html#multi-line-format");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
