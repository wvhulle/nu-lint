use nu_protocol::Span;

use super::{extract_int_value, get_slice_range, is_negative_expression};
use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

struct SliceToSkip;

enum SkipCount {
    Constant(i64),
}

struct FixData {
    slice_span: Span,
    skip_count: SkipCount,
}

impl DetectFix for SliceToSkip {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "slice_to_skip"
    }

    fn explanation(&self) -> &'static str {
        "Use 'skip' instead of 'slice N..' to skip first N elements"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/skip.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Some(range) = get_slice_range(expr, ctx) else {
                return vec![];
            };

            // Pattern: slice N.. (has from, no to)
            if range.from.is_some() && range.to.is_none() && range.next.is_none() {
                let Some(from_expr) = &range.from else {
                    return vec![];
                };

                // Use AST to skip negative values (handled by slice_to_last)
                if is_negative_expression(from_expr, ctx) {
                    return vec![];
                }

                // Only handle constants - variables produce Garbage in Nu parser
                let Some(n) = extract_int_value(from_expr, ctx) else {
                    return vec![];
                };

                // Skip 0 (slice 0.. is a no-op)
                if n == 0 {
                    return vec![];
                }

                let detection = Detection::from_global_span(
                    "Use 'skip' instead of 'slice' to skip first elements".to_string(),
                    expr.span,
                )
                .with_primary_label("slice command");

                let fix_data = FixData {
                    slice_span: expr.span,
                    skip_count: SkipCount::Constant(n),
                };

                vec![(detection, fix_data)]
            } else {
                vec![]
            }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = match &fix_data.skip_count {
            SkipCount::Constant(n) => format!("skip {n}"),
        };

        Some(Fix::with_explanation(
            "Replace with 'skip'",
            vec![Replacement::new(fix_data.slice_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &SliceToSkip;
