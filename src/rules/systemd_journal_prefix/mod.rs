use nu_protocol::ast::{Expr, Expression};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

const VALID_NUMERIC_PREFIXES: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];
const VALID_KEYWORD_PREFIXES: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];

fn detect_keyword_in_message(text: &str) -> &'static str {
    let lower = text.to_lowercase();

    let first_word = lower
        .split(|c: char| c.is_whitespace() || c == ':')
        .find(|w| !w.is_empty())
        .unwrap_or("");

    match first_word {
        "emergency" | "emerg" | "panic" => "emerg",
        "alert" => "alert",
        "critical" | "crit" | "fatal" => "crit",
        "error" | "err" | "fail" | "failed" => "err",
        "warning" | "warn" | "caution" => "warning",
        "notice" => "notice",
        "info" | "information" => "info",
        "debug" | "trace" => "debug",
        _ => {
            if lower.contains("error") || lower.contains("fail") {
                "err"
            } else if lower.contains("warn") {
                "warning"
            } else if lower.contains("debug") {
                "debug"
            } else {
                "info"
            }
        }
    }
}

fn has_journal_prefix(text: &str) -> bool {
    text.trim_start()
        .strip_prefix('<')
        .and_then(|s| s.split_once('>'))
        .is_some_and(|(prefix, _)| {
            VALID_NUMERIC_PREFIXES.contains(&prefix) || VALID_KEYWORD_PREFIXES.contains(&prefix)
        })
}

fn extract_first_string_part(expr: &Expression, ctx: &LintContext) -> Option<String> {
    match &expr.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        Expr::StringInterpolation(parts) => parts.first().and_then(|first| match &first.expr {
            Expr::String(s) => Some(s.clone()),
            _ => None,
        }),
        _ => Some(expr.span_text(ctx).to_string()),
    }
}

fn generate_fixed_string(
    original: &str,
    suggested_level: &str,
    arg_expr: &Expression,
    ctx: &LintContext,
) -> String {
    match &arg_expr.expr {
        Expr::String(_) => format!("\"<{suggested_level}>{original}\""),
        Expr::RawString(_) => format!("'<{suggested_level}>{original}'"),
        Expr::StringInterpolation(_) => format!("$\"<{suggested_level}>{original}\""),
        _ => {
            let original_text = arg_expr.span_text(ctx);
            format!("\"<{suggested_level}>{original_text}\"")
        }
    }
}

fn create_violation(
    command_name: &str,
    span: nu_protocol::Span,
    message_text: &str,
    arg_expr: &Expression,
    ctx: &LintContext,
) -> Violation {
    let suggested_level = detect_keyword_in_message(message_text);
    let example = format!("{command_name} \"<{suggested_level}>{message_text}\"");

    let fixed_string = generate_fixed_string(message_text, suggested_level, arg_expr, ctx);
    let fix = Fix::with_explanation(
        format!(
            "Add '<{suggested_level}>' prefix to the message:\n  {command_name} {fixed_string}"
        ),
        vec![Replacement::new(arg_expr.span, fixed_string)],
    );

    Violation::new(
        "systemd_journal_prefix",
        "Output without systemd journal log level prefix - consider adding prefix for proper \
         logging",
        span,
    )
    .with_help(format!(
        "Add systemd journal prefix using keywords: <emerg>, <alert>, <crit>, <err>, <warning>, \
         <notice>, <info>, <debug>. Suggested level for this message: <{suggested_level}>. \
         Example: {example}"
    ))
    .with_fix(fix)
}

fn check_print_or_echo_call(expr: &Expression, ctx: &LintContext) -> Vec<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let command_name = call.get_call_name(ctx);
    if !matches!(command_name.as_str(), "print" | "echo") {
        return vec![];
    }

    let Some(arg_expr) = call.get_first_positional_arg() else {
        return vec![];
    };

    let Some(message_content) = extract_first_string_part(arg_expr, ctx) else {
        return vec![];
    };

    if has_journal_prefix(&message_content) {
        return vec![];
    }

    vec![create_violation(
        &command_name,
        expr.span,
        &message_content,
        arg_expr,
        ctx,
    )]
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
mod generated_fix;
#[cfg(test)]
mod ignore_good;
