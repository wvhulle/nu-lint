use nu_protocol::ast::{Expr, Expression, RecordItem, Traverse};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MAX_RECORD_LINE_LENGTH: usize = 80;

#[derive(Clone)]
enum RecordFieldData {
    Pair {
        key_span: nu_protocol::Span,
        value_span: nu_protocol::Span,
    },
    Spread {
        spread_span: nu_protocol::Span,
    },
}

struct RecordFixData {
    span: nu_protocol::Span,
    fields: Vec<RecordFieldData>,
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

fn create_violation(span: nu_protocol::Span, fields: &[RecordItem]) -> (Detection, RecordFixData) {
    let field_data = fields
        .iter()
        .map(|item| match item {
            RecordItem::Pair(key, value) => RecordFieldData::Pair {
                key_span: key.span,
                value_span: value.span,
            },
            RecordItem::Spread(spread_span, expr) => RecordFieldData::Spread {
                spread_span: nu_protocol::Span::new(spread_span.start, expr.span.end),
            },
        })
        .collect();

    (
        Detection::from_global_span(
            "Long records should use multiline format with each field on a separate line",
            span,
        ),
        RecordFixData {
            span,
            fields: field_data,
        },
    )
}

/// This rule uses AST-based detection and is compatible with topiary-nushell
/// tree-sitter formatting. It analyzes actual record structures rather than
/// regex patterns.
struct WrapWideRecords;

impl DetectFix for WrapWideRecords {
    type FixInput<'a> = RecordFixData;

    fn id(&self) -> &'static str {
        "wrap_wide_records"
    }

    fn explanation(&self) -> &'static str {
        "Prefer multiline format for long or complex records"
    }

    fn doc_url(&self) -> Option<&'static str> {
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
                if let Expr::Record(fields) = &expr.expr {
                    if should_be_multiline(expr, fields, context) {
                        vec![create_violation(expr.span, fields)]
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
        let mut result = String::from("{\n");

        for field in &fix_data.fields {
            result.push_str("    ");
            match field {
                RecordFieldData::Pair {
                    key_span,
                    value_span,
                } => {
                    let key_text = context.plain_text(*key_span);
                    let value_text = context.plain_text(*value_span);
                    result.push_str(key_text);
                    result.push_str(": ");
                    result.push_str(value_text);
                }
                RecordFieldData::Spread { spread_span } => {
                    let spread_text = context.plain_text(*spread_span);
                    result.push_str(spread_text);
                }
            }
            result.push('\n');
        }

        result.push('}');

        Some(Fix::with_explanation(
            "Wrap record fields on separate lines",
            vec![Replacement::new(fix_data.span, result)],
        ))
    }
}

pub static RULE: &dyn Rule = &WrapWideRecords;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
