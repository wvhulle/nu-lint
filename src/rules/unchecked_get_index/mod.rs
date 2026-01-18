use nu_protocol::{
    Span,
    ast::{Expr, Expression, PathMember},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, span::SpanExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct GetIndexFixData {
    insert_span: Span,
}

fn check_get_call(
    expr: &Expression,
    try_block_spans: &[Span],
    context: &LintContext,
) -> Option<(Detection, GetIndexFixData)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("get", context) {
        return None;
    }

    // If already using -o flag, it's safe
    if call.has_named_flag("optional") || call.has_named_flag("o") {
        return None;
    }

    let key_arg = call.get_first_positional_arg()?;

    // Check if the argument is a numeric index (accessing by index)
    let is_numeric_access = match &key_arg.expr {
        Expr::Int(_) => true,
        Expr::CellPath(cp) => cp
            .members
            .iter()
            .any(|m| matches!(m, PathMember::Int { .. })),
        _ => false,
    };

    if !is_numeric_access {
        return None;
    }

    if expr.span.is_inside_any(try_block_spans) {
        return None;
    }

    let get_keyword_end = call.head.end;
    let insert_span = Span::new(get_keyword_end, get_keyword_end);

    let violation = Detection::from_global_span(
        "List access with 'get' without -o flag may panic if index is out of bounds",
        call.head,
    )
    .with_primary_label("add -o flag for safe access")
    .with_extra_label("numeric index", key_arg.span);

    let fix_data = GetIndexFixData { insert_span };

    Some((violation, fix_data))
}

struct UncheckedGetIndex;

impl DetectFix for UncheckedGetIndex {
    type FixInput<'a> = GetIndexFixData;

    fn id(&self) -> &'static str {
        "unchecked_get_index"
    }

    fn short_description(&self) -> &'static str {
        "List access with 'get' requires -o flag for safety"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Using 'get' with numeric indices without the -o (optional) flag causes a panic if \
             the index is out of bounds. Add -o to return null instead of panicking.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/get.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let try_block_spans = context.collect_command_spans(&["try"]);
        context.detect_with_fix_data(|expr, ctx| {
            check_get_call(expr, &try_block_spans, ctx)
                .into_iter()
                .collect()
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Add -o flag for safe optional access".into(),
            replacements: vec![Replacement::new(fix_data.insert_span, " -o")],
        })
    }
}

pub static RULE: &dyn Rule = &UncheckedGetIndex;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
