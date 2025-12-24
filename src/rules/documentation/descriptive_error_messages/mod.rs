use nu_protocol::ast::{Call, Expr, RecordItem};

use crate::{
    LintLevel, ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation,
};

const VAGUE_PATTERNS: &[&str] = &[
    "error",
    "failed",
    "failure",
    "err",
    "something went wrong",
    "an error occurred",
    "unknown error",
    "unexpected error",
    "operation failed",
    "invalid",
    "bad",
    "wrong",
];

const MIN_MEANINGFUL_LENGTH: usize = 15;

fn is_vague_message(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    if trimmed.len() < MIN_MEANINGFUL_LENGTH {
        return true;
    }

    VAGUE_PATTERNS
        .iter()
        .any(|&pattern| trimmed == pattern || trimmed.starts_with(&format!("{pattern}:")))
}

fn check_record_msg_field(record: &[RecordItem]) -> Option<Violation> {
    record.iter().find_map(|item| {
        let RecordItem::Pair(key, value) = item else {
            return None;
        };

        if key.as_string() != Some("msg".into()) {
            return None;
        }

        let msg = value.as_string()?;
        is_vague_message(&msg).then(|| {
            Violation::new("Error message is too vague or generic", value.span)
                .with_primary_label("vague message")
                .with_help(
                    "Use a descriptive message explaining what went wrong and how to fix it.",
                )
        })
    })
}

fn check_print_stderr(call: &Call) -> Option<Violation> {
    if !call.has_named_flag("stderr") && !call.has_named_flag("e") {
        return None;
    }

    let first_arg = call.get_first_positional_arg()?;
    let msg = first_arg.as_string()?;

    is_vague_message(&msg).then(|| {
        Violation::new(
            "Error message printed to stderr is too vague",
            first_arg.span,
        )
        .with_primary_label("vague error output")
        .with_help("Use a descriptive message explaining what went wrong and how to fix it.")
    })
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        let name = call.get_call_name(ctx);

        match name.as_str() {
            "error make" => {
                let Some(first_arg) = call.get_first_positional_arg() else {
                    return vec![];
                };

                let record = match &first_arg.expr {
                    Expr::Record(r) => r,
                    Expr::FullCellPath(cp) => match &cp.head.expr {
                        Expr::Record(r) => r,
                        _ => return vec![],
                    },
                    _ => return vec![],
                };

                check_record_msg_field(record).into_iter().collect()
            }
            "print" => check_print_stderr(call).into_iter().collect(),
            _ => vec![],
        }
    })
}

pub const RULE: Rule = Rule::new(
    "descriptive_error_messages",
    "Error messages should be descriptive and actionable",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/creating_errors.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
