use nu_protocol::ast::{Expr, Expression, ListItem, Traverse};

use crate::{
    LintLevel, ast::expression::ExpressionExt, context::LintContext, rule::Rule,
    violation::Violation,
};

const MAX_LIST_LINE_LENGTH: usize = 80;

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::List(items) = &expr.expr {
                if should_be_multiline(expr, items, context) {
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

fn should_be_multiline(expr: &Expression, items: &[ListItem], context: &LintContext) -> bool {
    let text = expr.span_text(context);

    if text.contains('\n') {
        return false;
    }

    // Should be multiline if:
    // 1. Single line AND more than 80 characters, OR
    // 2. Single line AND contains nested lists or records
    text.len() > MAX_LIST_LINE_LENGTH || has_nested_structures(items)
}

fn has_nested_structures(items: &[ListItem]) -> bool {
    items.iter().any(|item| {
        let expr = match item {
            ListItem::Item(e) | ListItem::Spread(_, e) => e,
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
        "Long lists should use multiline format with each item on a separate line",
        span,
    )
    .with_help("Put each list item on a separate line for better readability")
}

/// This rule uses AST-based detection and is compatible with topiary-nushell
/// tree-sitter formatting. It analyzes actual list structures rather than regex
/// patterns.
pub const RULE: Rule = Rule::new(
    "reflow_wide_lists",
    "Wrap wide lists vertically across multiple lines.",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/style_guide.html#multi-line-format");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
