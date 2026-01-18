use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_first_last_call(
    expr: &Expression,
    try_block_spans: &[Span],
    context: &LintContext,
) -> Option<Detection> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let cmd_name = call.get_call_name(context);

    if !matches!(cmd_name.as_str(), "first" | "last") {
        return None;
    }

    // first/last without arguments returns single item and can error on empty
    // first N / last N returns list and doesn't error
    if call.get_first_positional_arg().is_some() {
        return None;
    }

    if expr.span.is_inside_any(try_block_spans) {
        return None;
    }

    let violation = Detection::from_global_span(
        format!(
            "'{cmd_name}' without count argument may panic if list is empty, use 'try' or check \
             'is-empty' first"
        ),
        call.head,
    )
    .with_primary_label("unchecked access");

    Some(violation)
}

struct UncheckedFirstLast;

impl DetectFix for UncheckedFirstLast {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "unchecked_first_last"
    }

    fn short_description(&self) -> &'static str {
        "Using 'first' or 'last' without count may panic on empty lists"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Calling 'first' or 'last' without a count argument on an empty list causes a runtime \
             panic. Consider using 'first 1' or 'last 1' which returns an empty list instead of \
             panicking, wrapping in a 'try' block, or checking with 'is-empty' first.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let try_block_spans = context.collect_command_spans(&["try"]);
        Self::no_fix(context.detect(|expr, ctx| {
            check_first_last_call(expr, &try_block_spans, ctx)
                .into_iter()
                .collect()
        }))
    }
}

pub static RULE: &dyn Rule = &UncheckedFirstLast;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
