use nu_protocol::ast::{Expr, Expression, RecordItem, Traverse};

use crate::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const MAX_RECORD_LINE_LENGTH: usize = 80;
const NESTED_STRUCTURE_LENGTH_THRESHOLD: usize = 60;

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

    let len = text.len();
    if len > MAX_RECORD_LINE_LENGTH {
        return true;
    }

    let depth = max_nesting_depth(fields);
    depth > 1 || (depth == 1 && len > NESTED_STRUCTURE_LENGTH_THRESHOLD)
}

/// Returns the maximum nesting depth of structures within record fields.
/// 0 = no nested structures, 1 = has nested record/list, 2+ = deeply nested
fn max_nesting_depth(fields: &[RecordItem]) -> usize {
    fields
        .iter()
        .map(|item| {
            let expr = match item {
                RecordItem::Pair(_, val) => val,
                RecordItem::Spread(_, expr) => expr,
            };
            expr_nesting_depth(expr)
        })
        .max()
        .unwrap_or(0)
}

fn expr_nesting_depth(expr: &Expression) -> usize {
    use nu_protocol::ast::ListItem;

    let inner = match &expr.expr {
        Expr::FullCellPath(fcp) => &fcp.head.expr,
        other => other,
    };

    match inner {
        Expr::Record(fields) => 1 + max_nesting_depth(fields),
        Expr::List(items) => {
            let max_item_depth = items
                .iter()
                .map(|item| {
                    let e = match item {
                        ListItem::Item(e) | ListItem::Spread(_, e) => e,
                    };
                    expr_nesting_depth(e)
                })
                .max()
                .unwrap_or(0);
            1 + max_item_depth
        }
        _ => 0,
    }
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

    fn short_description(&self) -> &'static str {
        "Wrap records exceeding 80 chars or with deeply nested structures"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Records should use multiline format when they exceed 80 characters, or when they \
             contain nested structures and exceed 60 characters, or when they have deeply nested \
             structures (records/lists inside records/lists).",
        )
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

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        use std::fmt::Write;

        let mut fields = String::new();
        for field in &fix_data.fields {
            let field_text = match field {
                RecordFieldData::Pair {
                    key_span,
                    value_span,
                } => format!(
                    "{}: {}",
                    ctx.span_text(*key_span),
                    ctx.span_text(*value_span)
                ),
                RecordFieldData::Spread { spread_span } => ctx.span_text(*spread_span).to_string(),
            };
            let _ = writeln!(fields, "    {field_text}");
        }

        let result = format!("{{\n{fields}}}");

        Some(Fix {
            explanation: "Wrap record fields on separate lines".into(),
            replacements: vec![Replacement::new(fix_data.span, result)],
        })
    }
}

pub static RULE: &dyn Rule = &WrapWideRecords;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
