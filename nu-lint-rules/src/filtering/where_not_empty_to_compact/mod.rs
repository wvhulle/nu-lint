use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::ast::{Expr, Expression};

use super::where_it_not_equal_matches;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;

fn detect_empty_string_filter(where_expr: &Expression, context: &LintContext) -> Option<Detection> {
    let Expr::Call(where_call) = &where_expr.expr else {
        return None;
    };

    where_it_not_equal_matches(
        where_call,
        context,
        |compared_operand| matches!(&compared_operand.expr, Expr::String(text) if text.is_empty()),
    )
    .then(|| {
        Detection::from_global_span(
            "Use `compact --empty` if null, empty lists and empty records should also be dropped",
            where_expr.span,
        )
        .with_primary_label("may be replaceable with `compact --empty`")
    })
}

struct WhereNotEmptyToCompact;

impl DetectFix for WhereNotEmptyToCompact {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "where_not_empty_to_compact"
    }

    fn short_description(&self) -> &'static str {
        "Empty string filter may be replaceable with `compact --empty`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Dropping empty strings with `where $it != \"\"` is usually meant to remove every \
             empty value, which is what `compact --empty` does. This is not offered as an \
             automatic fix because `compact --empty` also drops null, `[]` and `{}`, which the \
             filter keeps.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/compact.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect_single(detect_empty_string_filter))
    }
}

pub static RULE: &dyn Rule = &WhereNotEmptyToCompact;
