use nu_protocol::ast::{Expr, Expression};

use crate::{
    ast_utils::AstUtils,
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

const GENERIC_ERROR_MESSAGES: &[&str] = &[
    "error",
    "failed",
    "err",
    "something went wrong",
];

/// Check if a string literal contains a generic error message
fn is_generic_error_message(text: &str) -> bool {
    let lower_text = text.to_lowercase();
    GENERIC_ERROR_MESSAGES.iter().any(|&generic| lower_text == generic)
}

/// Extract string literal from an expression if it's a string
fn extract_string_literal(expr: &Expression, context: &LintContext) -> Option<String> {
    match &expr.expr {
        Expr::String(s) => Some(s.clone()),
        Expr::RawString(s) => Some(s.clone()),
        _ => {
            // Fallback to span text for other string representations
            let text = AstUtils::span_text(expr.span, context);
            if (text.starts_with('"') && text.ends_with('"')) ||
               (text.starts_with('\'') && text.ends_with('\'')) {
                Some(text[1..text.len()-1].to_string())
            } else {
                None
            }
        }
    }
}

/// Check a record for generic error messages in msg field
fn check_record_for_generic_msg(record: &Vec<nu_protocol::ast::RecordItem>, context: &LintContext) -> Option<RuleViolation> {
    for item in record {
        if let nu_protocol::ast::RecordItem::Pair(key, value) = item {
            // Extract field name from key expression
            let field_name = match &key.expr {
                Expr::String(s) => s.clone(),
                Expr::RawString(s) => s.clone(),
                _ => {
                    let text = AstUtils::span_text(key.span, context);
                    if (text.starts_with('"') && text.ends_with('"')) ||
                       (text.starts_with('\'') && text.ends_with('\'')) {
                        text[1..text.len()-1].to_string()
                    } else {
                        text.to_string()
                    }
                }
            };

            if field_name == "msg" {
                if let Some(msg_text) = extract_string_literal(value, context) {
                    if is_generic_error_message(&msg_text) {
                        return Some(
                            RuleViolation::new_static(
                                "descriptive_error_messages",
                                "Error message is too generic and not descriptive",
                                value.span,
                            )
                            .with_suggestion_static(
                                "Use a descriptive error message that explains what went wrong and how to fix it.\nExample: error make { msg: \"Failed to parse input: expected number, got string\" }",
                            ),
                        );
                    }
                }
            }
        }
    }
    None
}

fn check_error_make_call(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<RuleViolation> {
    let decl_name = AstUtils::get_call_name(call, context);

    if decl_name != "error make" {
        return None;
    }

    // Check the first argument which should be a record
    let first_arg = AstUtils::get_first_positional_arg(call)?;

    match &first_arg.expr {
        Expr::Record(record) => check_record_for_generic_msg(record, context),
        Expr::FullCellPath(cell_path) => {
            // Handle case where record is wrapped in FullCellPath
            match &cell_path.head.expr {
                Expr::Record(record) => check_record_for_generic_msg(record, context),
                _ => None,
            }
        }
        _ => None,
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            check_error_make_call(call, ctx).into_iter().collect()
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "descriptive_error_messages",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Error messages should be descriptive and actionable",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
