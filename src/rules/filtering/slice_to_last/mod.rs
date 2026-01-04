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

struct SliceToLast;

enum LastCount {
    Constant(i64),
}

struct FixData {
    slice_span: Span,
    last_count: LastCount,
}

impl DetectFix for SliceToLast {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "slice_to_last"
    }

    fn explanation(&self) -> &'static str {
        "Use 'last' instead of 'slice (-N)..' to get last N elements"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/last.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Some(range) = get_slice_range(expr, ctx) else {
                return vec![];
            };

            // Pattern: slice (-N).. (from is negative, no to, no step)
            if range.from.is_none() || range.to.is_some() || range.next.is_some() {
                return vec![];
            }

            let Some(from_expr) = range.from.as_ref() else {
                return vec![];
            };

            // Use AST to check if it's negative
            if !is_negative_expression(from_expr, ctx) {
                return vec![];
            }

            // Only handle constants - variables produce Garbage in Nu parser
            let Some(n) = extract_int_value(from_expr, ctx) else {
                return vec![];
            };

            let detection = Detection::from_global_span(
                "Use 'last' instead of 'slice' to get last elements".to_string(),
                expr.span,
            )
            .with_primary_label("slice command");

            let fix_data = FixData {
                slice_span: expr.span,
                last_count: LastCount::Constant(-n), // n is negative, so -n is positive
            };

            vec![(detection, fix_data)]
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = match &fix_data.last_count {
            LastCount::Constant(n) => format!("last {n}"),
        };

        Some(Fix::with_explanation(
            "Replace with 'last'",
            vec![Replacement::new(fix_data.slice_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &SliceToLast;
