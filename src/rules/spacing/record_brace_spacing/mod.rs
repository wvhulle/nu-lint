use std::collections::HashSet;

use nu_protocol::{Span, ast::Expr};

use super::is_record_type;
use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    span::LintSpan,
    violation::{Detection, Fix, Replacement},
};

struct RecordBraceSpacingFixData {
    record_span: Span,
}

fn check_record_brace_spacing(
    context: &LintContext,
    record_span: Span,
) -> Vec<(Detection, RecordBraceSpacingFixData)> {
    let text = context.plain_text(record_span);

    // Validate basic structure using char iterators for UTF-8 safety
    let mut chars = text.chars();
    if chars.next() != Some('{') || chars.next_back() != Some('}') {
        return vec![];
    }

    let inner: String = chars.collect();
    if inner.trim().is_empty() {
        return vec![];
    }

    // Skip multiline records - they have different formatting conventions
    if inner.contains('\n') {
        return vec![];
    }

    let starts_with_space = inner.starts_with(char::is_whitespace);
    let ends_with_space = inner.ends_with(char::is_whitespace);

    if starts_with_space || ends_with_space {
        let opening_span = Span::new(record_span.start, record_span.start + 1);
        let closing_span = Span::new(record_span.end - 1, record_span.end);
        vec![(
            Detection::from_global_span(
                "Record braces should touch content: `{a: 1}` not `{ a: 1 }`".to_string(),
                record_span,
            )
            .with_extra_label("remove space after `{`", opening_span)
            .with_extra_label("remove space before `}`", closing_span),
            RecordBraceSpacingFixData { record_span },
        )]
    } else {
        vec![]
    }
}

struct RecordBraceSpacing;

impl DetectFix for RecordBraceSpacing {
    type FixInput<'a> = RecordBraceSpacingFixData;

    fn id(&self) -> &'static str {
        "record_brace_spacing"
    }

    fn explanation(&self) -> &'static str {
        "Record braces should touch content: `{a: 1}` not `{ a: 1 }`"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/style_guide.html#one-line-format")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut seen_spans: HashSet<(usize, usize)> = HashSet::new();
        let results = context.detect_with_fix_data(|expr, ctx| {
            match &expr.expr {
                // Nushell parses record literals in variable assignments as Block with Record type
                Expr::Block(_) if is_record_type(&expr.ty) => {
                    check_record_brace_spacing(ctx, expr.span)
                }
                Expr::Record(items) if !items.is_empty() => {
                    check_record_brace_spacing(ctx, expr.span)
                }
                _ => vec![],
            }
        });

        // Deduplicate by span - the same record can be visited via multiple AST paths
        results
            .into_iter()
            .filter(|(detection, _)| {
                let span_key = match detection.span {
                    LintSpan::Global(s) => (s.start, s.end),
                    LintSpan::File(s) => (s.start, s.end),
                };
                seen_spans.insert(span_key)
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let text = context.plain_text(fix_data.record_span);

        // Extract inner content using char iterators for UTF-8 safety
        let mut chars = text.chars();
        if chars.next() != Some('{') || chars.next_back() != Some('}') {
            return None;
        }
        let inner: String = chars.collect();
        let trimmed = inner.trim();
        let fixed = format!("{{{trimmed}}}");

        Some(Fix::with_explanation(
            "Remove spaces inside record braces",
            vec![Replacement::new(fix_data.record_span, fixed)],
        ))
    }
}

pub static RULE: &dyn Rule = &RecordBraceSpacing;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
