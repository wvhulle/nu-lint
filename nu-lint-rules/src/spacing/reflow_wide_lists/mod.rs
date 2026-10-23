use nu_protocol::ast::{Expr, Expression, ListItem, Traverse};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MAX_LIST_LINE_LENGTH: usize = 80;

struct ListFixData {
    span: nu_protocol::Span,
    items: Vec<nu_protocol::Span>,
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

fn create_violation(span: nu_protocol::Span, items: &[ListItem]) -> (Detection, ListFixData) {
    let item_spans = items
        .iter()
        .map(|item| match item {
            ListItem::Item(e) | ListItem::Spread(_, e) => e.span,
        })
        .collect();

    (
        Detection::from_global_span(
            "Long lists should use multiline format with each item on a separate line",
            span,
        ),
        ListFixData {
            span,
            items: item_spans,
        },
    )
}

/// This rule uses AST-based detection and is compatible with topiary-nushell
/// tree-sitter formatting. It analyzes actual list structures rather than regex
/// patterns.
struct ReflowWideLists;

impl DetectFix for ReflowWideLists {
    type FixInput<'a> = ListFixData;

    fn id(&self) -> &'static str {
        "reflow_wide_lists"
    }

    fn short_description(&self) -> &'static str {
        "Wrap wide lists vertically across multiple lines."
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#multi-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| {
                if let Expr::List(items) = &expr.expr {
                    if should_be_multiline(expr, items, context) {
                        vec![create_violation(expr.span, items)]
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let mut result = String::from("[\n");

        for item_span in &fix_data.items {
            let item_text = context.span_text(*item_span);
            result.push_str("    ");
            result.push_str(item_text);
            result.push('\n');
        }

        result.push(']');

        Some(Fix {
            explanation: "Wrap list items on separate lines".into(),
            replacements: vec![Replacement::new(fix_data.span, result)],
        })
    }
}

pub static RULE: &dyn Rule = &ReflowWideLists;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
