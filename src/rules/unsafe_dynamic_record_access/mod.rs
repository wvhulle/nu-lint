use nu_protocol::ast::{Expr, Expression};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

const fn is_dynamic_key(expr: &Expression) -> bool {
    matches!(
        &expr.expr,
        Expr::Var(_)
            | Expr::FullCellPath(_)
            | Expr::StringInterpolation(..)
            | Expr::Subexpression(_)
    )
}

fn check_get_call(expr: &Expression, ctx: &LintContext) -> Option<Violation> {
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
    let fix = Fix::with_explanation(
        "Add -o flag for safe optional access",
        vec![Replacement::new(
            nu_protocol::Span::new(get_keyword_end, get_keyword_end),
            " -o",
        )],
    );

    Some(
        Violation::new(
            "Dynamic record access without -o flag may silently fail",
            call.head,
        )
        .with_primary_label("add -o flag for safe access")
        .with_extra_label("dynamic key", key_arg.span)
        .with_help("Use 'get -o $key' to return null for missing keys instead of causing errors")
        .with_fix(fix),
    )
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| check_get_call(expr, ctx).into_iter().collect())
}

pub const fn rule() -> Rule {
    Rule::new(
        "unsafe_dynamic_record_access",
        "Detect 'get' calls with dynamic keys that don't use the -o/--optional flag, which can \
         cause silent failures when the key doesn't exist",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/get.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
