use nu_protocol::ast::{Call, Expr, RecordItem};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

const VAGUE_PATTERNS: &[&str] = &[
    "something went wrong",
    "fatal error",
    "does not work",
    "broke down",
    "an error occurred",
    "unknown error",
    "unexpected error",
    "operation failed",
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

fn check_record_msg_field(record: &[RecordItem]) -> Option<Detection> {
    record.iter().find_map(|item| {
        let RecordItem::Pair(key, value) = item else {
            return None;
        };

        if key.as_string() != Some("msg".into()) {
            return None;
        }

        let msg = value.as_string()?;
        is_vague_message(&msg).then(|| {
            Detection::from_global_span("Error message is too vague or generic", value.span)
                .with_primary_label("vague message")
        })
    })
}

fn check_print_stderr(call: &Call) -> Option<Detection> {
    if !call.has_named_flag("stderr") && !call.has_named_flag("e") {
        return None;
    }

    let first_arg = call.get_first_positional_arg()?;
    let msg = first_arg.as_string()?;

    is_vague_message(&msg).then(|| {
        Detection::from_global_span(
            "Error message printed to stderr is too vague",
            first_arg.span,
        )
        .with_primary_label("vague error output")
    })
}

fn check_error_make(call: &Call) -> Option<Detection> {
    let first_arg = call.get_first_positional_arg()?;

    let record = match &first_arg.expr {
        Expr::Record(r) => r,
        Expr::FullCellPath(cp) => match &cp.head.expr {
            Expr::Record(r) => r,
            _ => return None,
        },
        _ => return None,
    };

    check_record_msg_field(record)
}

struct DescriptiveErrorMessages;

impl DetectFix for DescriptiveErrorMessages {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "descriptive_error_messages"
    }

    fn explanation(&self) -> &'static str {
        "Error messages should be descriptive and actionable"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/creating_errors.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            match call.get_call_name(ctx).as_str() {
                "error make" => check_error_make(call).into_iter().collect(),
                "print" => check_print_stderr(call).into_iter().collect(),
                _ => vec![],
            }
        }))
    }
}

pub static RULE: &dyn Rule = &DescriptiveErrorMessages;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
