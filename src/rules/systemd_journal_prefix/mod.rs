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
        _ => detect_keyword_in_full_text(&lower),
    }
}

fn detect_keyword_in_full_text(lower_text: &str) -> &'static str {
    if lower_text.contains("error") || lower_text.contains("fail") {
        "err"
    } else if lower_text.contains("warn") {
        "warning"
    } else if lower_text.contains("debug") {
        "debug"
    } else {
        "info"
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

fn strip_keyword_prefix(text: &str) -> &str {
    const KEYWORD_PREFIXES: &[&str] = &[
        "emergency:",
        "emerg:",
        "panic:",
        "alert:",
        "critical:",
        "crit:",
        "fatal:",
        "error:",
        "err:",
        "fail:",
        "failed:",
        "warning:",
        "warn:",
        "caution:",
        "notice:",
        "info:",
        "information:",
        "debug:",
        "trace:",
    ];

    let trimmed = text.trim_start();
    let lower = trimmed.to_lowercase();

    KEYWORD_PREFIXES
        .iter()
        .find(|&&keyword| lower.starts_with(keyword))
        .map_or(text, |&keyword| trimmed[keyword.len()..].trim_start())
}

fn generate_fixed_string(
    original: &str,
    suggested_level: &str,
    arg_expr: &Expression,
    ctx: &LintContext,
) -> String {
    let cleaned_message = strip_keyword_prefix(original);
    let span_text = arg_expr.span_text(ctx);

    match &arg_expr.expr {
        Expr::String(_) => format_string_with_quotes(cleaned_message, suggested_level, span_text),
        Expr::RawString(_) => format!("'<{suggested_level}>{cleaned_message}'"),
        Expr::StringInterpolation(_) => format_interpolated_string(suggested_level, span_text),
        _ => format!("\"<{suggested_level}>{span_text}\""),
    }
}

fn format_string_with_quotes(message: &str, level: &str, span_text: &str) -> String {
    let quote_char = if span_text.starts_with('\'') {
        '\''
    } else {
        '"'
    };
    format!("{quote_char}<{level}>{message}{quote_char}")
}

fn format_interpolated_string(level: &str, span_text: &str) -> String {
    span_text
        .strip_prefix("$\"")
        .and_then(|s| s.strip_suffix('"'))
        .map_or_else(
            || format!("$\"<{level}>{{original content}}\""),
            |content| {
                let cleaned = strip_keyword_prefix(content);
                format!("$\"<{level}>{cleaned}\"")
            },
        )
}

fn create_violation(
    command_name: &str,
    span: nu_protocol::Span,
    message_text: &str,
    arg_expr: &Expression,
    ctx: &LintContext,
) -> Violation {
    let suggested_level = detect_keyword_in_message(message_text);
    let fixed_string = generate_fixed_string(message_text, suggested_level, arg_expr, ctx);

    let help_message = format!(
        "Add systemd journal prefix using keywords: <emerg>, <alert>, <crit>, <err>, <warning>, \
         <notice>, <info>, <debug>. Suggested level for this message: <{suggested_level}>. \
         Example: {command_name} \"<{suggested_level}>{message_text}\""
    );

    let fix_explanation = format!(
        "Add '<{suggested_level}>' prefix to the message:\n  {command_name} {fixed_string}"
    );

    Violation::new(
        "systemd_journal_prefix",
        "Output without systemd journal log level prefix - consider adding prefix for proper \
         logging",
        span,
    )
    .with_help(help_message)
    .with_fix(Fix::with_explanation(
        fix_explanation,
        vec![Replacement::new(arg_expr.span, fixed_string)],
    ))
}

fn check_print_or_echo_call(expr: &Expression, ctx: &LintContext) -> Vec<Violation> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    let command_name = call.get_call_name(ctx);
    if !matches!(command_name.as_str(), "print" | "echo") {
        return vec![];
    }

    call.get_first_positional_arg()
        .and_then(|arg_expr| {
            extract_first_string_part(arg_expr, ctx).and_then(|message_content| {
                (!has_journal_prefix(&message_content)).then(|| {
                    create_violation(&command_name, expr.span, &message_content, arg_expr, ctx)
                })
            })
        })
        .into_iter()
        .collect()
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| check_print_or_echo_call(expr, ctx))
}
pub const fn rule() -> Rule {
    Rule::new(
        "systemd_journal_prefix",
        "Assign log levels to output that is compatible with the SystemD service option \
         SyslogLevelPrefix. This will allow proper categorization of log messages in the systemd \
         journal.",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
