use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use super::where_it_not_equal_matches;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

fn detect_null_filter(where_expr: &Expression, context: &LintContext) -> Vec<(Detection, Span)> {
    let Expr::Call(where_call) = &where_expr.expr else {
        return vec![];
    };

    if !where_it_not_equal_matches(where_call, context, |compared_operand| {
        matches!(compared_operand.expr, Expr::Nothing)
    }) {
        return vec![];
    }

    let detection = Detection::from_global_span(
        "Use `compact` to drop null values from a list",
        where_expr.span,
    )
    .with_primary_label("replaceable with `compact`");

    vec![(detection, where_expr.span)]
}

struct WhereNotNullToCompact;

impl DetectFix for WhereNotNullToCompact {
    type FixInput<'a> = Span;

    fn id(&self) -> &'static str {
        "where_not_null_to_compact"
    }

    fn short_description(&self) -> &'static str {
        "Simplify `where $it != null` to `compact`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The `compact` command exists to drop null values, and it returns the same result as \
             filtering each value with `where $it != null`, for lists and for tables.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/compact.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(detect_null_filter)
    }

    fn fix(&self, _context: &LintContext, where_span: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Drop null values with `compact`".into(),
            replacements: vec![Replacement::new(*where_span, "compact")],
        })
    }
}

pub static RULE: &dyn Rule = &WhereNotNullToCompact;
