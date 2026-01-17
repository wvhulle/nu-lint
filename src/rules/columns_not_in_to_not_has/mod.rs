use nu_protocol::{
    Span,
    ast::{Comparison, Expr, Expression, Operator},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, expression::ExpressionExt},
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

struct FixData {
    full: Span,
    key: Span,
    record: Span,
}

fn check_not_in_columns(expr: &Expression, ctx: &LintContext) -> Vec<(Detection, FixData)> {
    let Expr::BinaryOp(left, op, right) = &expr.expr else {
        return vec![];
    };

    let is_not_in_op = matches!(
        &op.expr,
        Expr::Operator(Operator::Comparison(Comparison::NotIn))
    );

    if !is_not_in_op {
        return vec![];
    }

    let Some(block_id) = right.extract_block_id() else {
        return vec![];
    };

    let block = ctx.working_set.get_block(block_id);
    let Some(record_span) = block.find_columns_record_span(ctx) else {
        return vec![];
    };

    let detection = Detection::from_global_span(
        "Use 'not-has' operator instead of 'not-in ($record | columns)' for record key membership",
        expr.span,
    )
    .with_primary_label("non-idiomatic key check")
    .with_extra_label("key", left.span)
    .with_extra_label("columns call", right.span);

    vec![(
        detection,
        FixData {
            full: expr.span,
            key: left.span,
            record: record_span,
        },
    )]
}

struct ColumnsNotInToNotHas;

impl DetectFix for ColumnsNotInToNotHas {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "columns_not_in_to_not_has"
    }

    fn short_description(&self) -> &'static str {
        "Simplify `not-in columns` to `not-has`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The pattern `$key not-in ($record | columns)` can be written more idiomatically as \
             `$record not-has $key`. The 'not-has' operator directly checks if a key is absent \
             from a record.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(check_not_in_columns)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let key_text = context.span_text(fix_data.key).trim();
        let record_text = context.span_text(fix_data.record).trim();

        let replacement = format!("{record_text} not-has {key_text}");

        Some(Fix::with_explanation(
            "Replace with 'not-has' operator",
            vec![Replacement::new(fix_data.full, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &ColumnsNotInToNotHas;
