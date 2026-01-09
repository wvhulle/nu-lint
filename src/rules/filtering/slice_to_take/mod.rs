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

struct SliceToTake;

enum TakeCount {
    Constant(i64),
}

struct FixData {
    slice_span: Span,
    take_count: TakeCount,
}

impl DetectFix for SliceToTake {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "slice_to_take"
    }

    fn short_description(&self) -> &'static str {
        "Use 'take' instead of 'slice 0..N' to take first N elements"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/take.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            let Some(range) = get_slice_range(expr, ctx) else {
                return vec![];
            };

            // Pattern: slice 0..N (from is 0, to exists, no step)
            if range.from.is_some() && range.to.is_some() && range.next.is_none() {
                let Some(from_expr) = &range.from else {
                    return vec![];
                };
                let Some(to_expr) = &range.to else {
                    return vec![];
                };

                // Check from is 0 using AST
                if extract_int_value(from_expr, ctx) != Some(0) {
                    return vec![];
                }

                // Use AST to skip negative values
                if is_negative_expression(to_expr, ctx) {
                    return vec![];
                }

                // Only handle constants - variables produce Garbage in Nu parser
                let Some(n) = extract_int_value(to_expr, ctx) else {
                    return vec![];
                };

                let detection = Detection::from_global_span(
                    "Use 'take' instead of 'slice 0..' to take first elements".to_string(),
                    expr.span,
                )
                .with_primary_label("slice command");

                let fix_data = FixData {
                    slice_span: expr.span,
                    take_count: TakeCount::Constant(n + 1),
                };

                vec![(detection, fix_data)]
            } else {
                vec![]
            }
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement_text = match &fix_data.take_count {
            TakeCount::Constant(n) => format!("take {n}"),
        };

        Some(Fix::with_explanation(
            "Replace with 'take'",
            vec![Replacement::new(fix_data.slice_span, replacement_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &SliceToTake;
