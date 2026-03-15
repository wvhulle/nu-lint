use nu_protocol::{
    Span, Type,
    ast::{Expr, Expression, Math, Operator},
};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    /// Span of the entire `$diff / 1hr` expression
    full: Span,
    /// Span of the left operand (`$diff`)
    left: Span,
    /// Span of the duration unit text (`hr`, `min`, `sec`)
    unit: Span,
}

fn extract_division_by_duration(expr: &Expression) -> Option<FixData> {
    let Expr::BinaryOp(left, op, rhs) = &expr.expr else {
        return None;
    };

    if !matches!(op.expr, Expr::Operator(Operator::Math(Math::Divide))) {
        return None;
    }

    if rhs.ty != Type::Duration {
        return None;
    }

    let Expr::ValueWithUnit(vu) = &rhs.expr else {
        return None;
    };

    Some(FixData {
        full: expr.span,
        left: left.span,
        unit: vu.unit.span,
    })
}

struct DivisionToFormatDuration;

impl DetectFix for DivisionToFormatDuration {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "division_to_format_duration"
    }

    fn short_description(&self) -> &'static str {
        "Replace duration division with `format duration`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Dividing by a duration literal (e.g. `$diff / 1hr`) manually extracts a time unit \
             from a duration. Use `format duration` for idiomatic unit conversion.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/format_duration.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, _ctx| {
            let Some(fix_data) = extract_division_by_duration(expr) else {
                return vec![];
            };

            let detection = Detection::from_global_span(
                "Duration division can be replaced with `format duration`",
                expr.span,
            )
            .with_primary_label("division by duration literal");

            vec![(detection, fix_data)]
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let left_text = context.span_text(fix_data.left);
        let unit_text = context.span_text(fix_data.unit);

        Some(Fix {
            explanation: "Use `format duration` instead of manual division".into(),
            replacements: vec![Replacement::new(
                fix_data.full,
                format!("{left_text} | format duration {unit_text}"),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &DivisionToFormatDuration;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
