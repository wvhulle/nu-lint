use nu_lint_analysis::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Type,
    ast::{Expr, Expression, Math, Operator},
};

fn is_division_by_duration_literal(division_expr: &Expression) -> bool {
    let Expr::BinaryOp(_, operator, divisor) = &division_expr.expr else {
        return false;
    };

    matches!(operator.expr, Expr::Operator(Operator::Math(Math::Divide)))
        && divisor.ty == Type::Duration
        && matches!(divisor.expr, Expr::ValueWithUnit(_))
}

struct DivisionToFormatDuration;

impl DetectFix for DivisionToFormatDuration {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "division_to_format_duration"
    }

    fn short_description(&self) -> &'static str {
        "Duration divided by a duration literal to get a number"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Dividing by a duration literal (e.g. `$diff / 1hr`) turns a duration into a plain \
             number and loses its unit. Compare against a duration literal (`$diff > 1hr`) when \
             checking a threshold, and use `format duration` when displaying the value. Neither \
             replaces the division automatically, because `format duration` returns a string.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/format_duration.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect_single(|division_expr, _context| {
            is_division_by_duration_literal(division_expr).then(|| {
                Detection::from_global_span(
                    "Compare against a duration literal, or use `format duration` for display",
                    division_expr.span,
                )
                .with_primary_label("division by duration literal")
            })
        }))
    }
}

pub static RULE: &dyn Rule = &DivisionToFormatDuration;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
