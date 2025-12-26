use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct DynamicAccessFixData {
    insert_span: nu_protocol::Span,
}

const fn is_dynamic_key(expr: &Expression) -> bool {
    matches!(
        &expr.expr,
        Expr::Var(_)
            | Expr::FullCellPath(_)
            | Expr::StringInterpolation(..)
            | Expr::Subexpression(_)
    )
}

fn check_get_call(
    expr: &Expression,
    ctx: &LintContext,
) -> Option<(Detection, DynamicAccessFixData)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("get", ctx) {
        return None;
    }

    if call.has_named_flag("optional") || call.has_named_flag("o") {
        return None;
    }

    let key_arg = call.get_first_positional_arg()?;

    if !is_dynamic_key(key_arg) {
        return None;
    }

    let get_keyword_end = call.head.end;
    let insert_span = nu_protocol::Span::new(get_keyword_end, get_keyword_end);

    let violation = Detection::from_global_span(
        "Dynamic record access without -o flag may silently fail",
        call.head,
    )
    .with_primary_label("add -o flag for safe access")
    .with_extra_label("dynamic key", key_arg.span)
    .with_help("Use 'get -o $key' to return null for missing keys instead of causing errors");

    let fix_data = DynamicAccessFixData { insert_span };

    Some((violation, fix_data))
}

struct UnsafeDynamicRecordAccess;

impl DetectFix for UnsafeDynamicRecordAccess {
    type FixInput<'a> = DynamicAccessFixData;

    fn id(&self) -> &'static str {
        "unsafe_dynamic_record_access"
    }

    fn explanation(&self) -> &'static str {
        "Use 'get -o' for dynamic keys to handle missing keys safely"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/get.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| check_get_call(expr, ctx).into_iter().collect())
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Add -o flag for safe optional access",
            vec![Replacement::new(fix_data.insert_span, " -o")],
        ))
    }
}

pub static RULE: &dyn Rule = &UnsafeDynamicRecordAccess;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
