use nu_protocol::ast::{Expr, Expression, RecordItem};

use super::{
    extract_field_name, extract_first_function_parameter, extract_record_from_expr,
    get_labels_value,
};
use crate::{
    LintLevel, ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation,
};

fn label_record_has_span(record: &[RecordItem], context: &LintContext) -> bool {
    record.iter().any(|item| {
        matches!(item, RecordItem::Pair(key, _) if extract_field_name(key, context) == "span")
    })
}

fn extract_label_record(expr: &Expression) -> Option<&[RecordItem]> {
    match &expr.expr {
        Expr::Record(record) => Some(record.as_slice()),
        Expr::FullCellPath(cell_path) => match &cell_path.head.expr {
            Expr::Record(record) => Some(record.as_slice()),
            _ => None,
        },
        _ => None,
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        if !call.is_call_to_command("error make", ctx) {
            return vec![];
        }

        call.get_first_positional_arg()
            .and_then(extract_record_from_expr)
            .and_then(|record| get_labels_value(record, ctx))
            .and_then(|labels_expr| {
                let label_record = extract_label_record(labels_expr)?;

                if label_record_has_span(label_record, ctx) {
                    return None;
                }

                let example_span = extract_first_function_parameter(ctx, call.span()).map_or_else(
                    || "$span".to_string(),
                    |param| format!("(metadata ${param}).span"),
                );

                Some(
                    Violation::new(
                        "labels field is missing 'span' to highlight error location in user code",
                        labels_expr.span,
                    )
                    .with_primary_label("missing span")
                    .with_help(format!(
                        "Add a 'span' field to point to the problematic code:\nlabels: {{ text: \
                         \"...\", span: {example_span} }}"
                    )),
                )
            })
            .into_iter()
            .collect()
    })
}

pub const RULE: Rule = Rule::new(
    "add_span_to_label",
    "labels field should include 'span' to highlight error location in user code",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/error_make.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
