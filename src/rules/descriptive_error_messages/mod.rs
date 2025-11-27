use nu_protocol::ast::{Call, Expr, Expression, RecordItem};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
};

const GENERIC_ERROR_MESSAGES: &[&str] = &["error", "failed", "err", "something went wrong"];

/// Check if a string literal contains a generic error message
fn is_generic_error_message(text: &str) -> bool {
    let lower_text = text.to_lowercase();
    GENERIC_ERROR_MESSAGES
        .iter()
        .any(|&generic| lower_text == generic)
}

/// Extract string literal from an expression if it's a string
fn extract_string_literal(expr: &Expression, context: &LintContext) -> Option<String> {
    match &expr.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        _ => {
            let text = expr.span_text(context);
            ((text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\'')))
            .then(|| text[1..text.len() - 1].to_string())
        }
    }
}

/// Extract field name from a record key expression
fn extract_field_name(key: &Expression, context: &LintContext) -> String {
    match &key.expr {
        Expr::String(s) | Expr::RawString(s) => s.clone(),
        _ => {
            let text = key.span_text(context);
            if (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('\'') && text.ends_with('\''))
            {
                text[1..text.len() - 1].to_string()
            } else {
                text.to_string()
            }
        }
    }
}

/// Check a record for generic error messages in msg field
fn check_record_for_generic_msg(
    record: &Vec<RecordItem>,
    context: &LintContext,
) -> Option<Violation> {
    for item in record {
        let RecordItem::Pair(key, value) = item else {
            continue;
        };

        let field_name = extract_field_name(key, context);

        if field_name != "msg" {
            continue;
        }

        let Some(msg_text) = extract_string_literal(value, context) else {
            continue;
        };

        if is_generic_error_message(&msg_text) {
            return Some(
                Violation::new(
                    "Error message is too generic and not descriptive",
                    value.span,
                )
                .with_help(
                    "Use a descriptive error message that explains what went wrong and how to fix \
                     it.\nExample: error make { msg: \"Failed to parse input: expected number, \
                     got string\" }",
                ),
            );
        }
    }
    None
}

fn check_error_make_call(call: &Call, context: &LintContext) -> Option<Violation> {
    let decl_name = call.get_call_name(context);

    if decl_name != "error make" {
        return None;
    }

    let first_arg = call.get_first_positional_arg()?;

    match &first_arg.expr {
        Expr::Record(record) => check_record_for_generic_msg(record, context),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Record(record) => check_record_for_generic_msg(record, context),
            _ => None,
        },
        _ => None,
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            check_error_make_call(call, ctx).into_iter().collect()
        } else {
            vec![]
        }
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "descriptive_error_messages",
        "Error messages should be descriptive and actionable",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/creating_errors.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
