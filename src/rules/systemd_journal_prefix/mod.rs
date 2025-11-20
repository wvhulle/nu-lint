use nu_protocol::ast::{Expr, Expression};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

const VALID_NUMERIC_PREFIXES: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];
const VALID_KEYWORD_PREFIXES: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];

fn has_journal_prefix(text: &str) -> bool {
    text.trim_start()
        .strip_prefix('<')
        .and_then(|s| s.split_once('>'))
        .is_some_and(|(prefix, _)| {
            VALID_NUMERIC_PREFIXES.contains(&prefix) || VALID_KEYWORD_PREFIXES.contains(&prefix)
        })
}

fn extract_first_string_part(expr: &Expression, context: &LintContext) -> Option<String> {
    match &expr.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        Expr::StringInterpolation(parts) => parts.first().and_then(|first| match &first.expr {
            Expr::String(s) => Some(s.clone()),
            _ => None,
        }),
        _ => Some(expr.span_text(context).to_string()),
    }
}

fn create_violation(command_name: &str, span: nu_protocol::Span) -> Violation {
    let example = format!("{command_name} \"<info>Starting process\"");

    Violation::new(
        "systemd_journal_prefix",
        "Output without systemd journal log level prefix - consider adding prefix for proper \
         logging",
        span,
    )
    .with_help(format!(
        "Add systemd journal prefix using numbers <0-7> or keywords: <emerg>, <alert>, <crit>, \
         <err>, <warning>, <notice>, <info>, <debug>. Example: {example}"
    ))
}

fn check_print_or_echo_call(expr: &Expression, context: &LintContext) -> Vec<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let command_name = call.get_call_name(context);
    if !matches!(command_name.as_str(), "print" | "echo") {
        return vec![];
    }

    call.get_first_positional_arg()
        .and_then(|arg| extract_first_string_part(arg, context))
        .filter(|content| !has_journal_prefix(content))
        .map(|_| vec![create_violation(&command_name, expr.span)])
        .unwrap_or_default()
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| check_print_or_echo_call(expr, ctx))
}
pub const fn rule() -> Rule {
    Rule::new(
        "systemd_journal_prefix",
        "Detect output without systemd journal log level prefix when using SyslogLevelPrefix",
        check,
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
