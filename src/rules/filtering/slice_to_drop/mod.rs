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

struct SliceToDrop;

enum DropCount {
    Constant(i64),
}

struct FixData {
    slice_span: Span,
    drop_count: DropCount,
}

impl DetectFix for SliceToDrop {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "slice_to_drop"
    }

    fn explanation(&self) -> &'static str {
        "Use 'drop' instead of 'slice ..-N' to drop last N-1 elements"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/drop.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Some(range) = get_slice_range(expr, ctx) else {
                return vec![];
            };

            // Pattern: slice ..-N or slice 0..-N (to is negative, no step)
            if range.to.is_none() || range.next.is_some() {
                return vec![];
            }

            let Some(to_expr) = &range.to else {
                return vec![];
            };

            // Use AST to check if it's negative
            if !is_negative_expression(to_expr, ctx) {
                return vec![];
            }

            // If from exists, check it's 0 using AST
            if range
                .from
                .as_ref()
                .is_some_and(|from_expr| extract_int_value(from_expr, ctx) != Some(0))
            {
                return vec![];
            }

            // Only handle constants - variables produce Garbage in Nu parser
            let Some(n) = extract_int_value(to_expr, ctx) else {
                return vec![];
            };

            // n is negative, so -n - 1 gives us the drop count
            let count = -n - 1;

            // Skip if drop count is 0 (no-op)
            if count == 0 {
                return vec![];
            }

            let detection = Detection::from_global_span(
                "Use 'drop' instead of 'slice' to drop last elements".to_string(),
                expr.span,
            )
            .with_primary_label("slice command");

            let fix_data = FixData {
                slice_span: expr.span,
                drop_count: DropCount::Constant(count),
            };

            vec![(detection, fix_data)]
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = match &fix_data.drop_count {
            DropCount::Constant(n) => format!("drop {n}"),
        };

        Some(Fix::with_explanation(
            "Replace with 'drop'",
            vec![Replacement::new(fix_data.slice_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &SliceToDrop;
