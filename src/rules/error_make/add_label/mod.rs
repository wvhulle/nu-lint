use nu_protocol::ast::Expr;

use super::{extract_first_function_parameter, extract_record_from_expr, has_field};
use crate::{
    LintLevel, ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation,
};

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
            .and_then(|record| {
                if !has_field(record, "msg", ctx) || has_field(record, "labels", ctx) {
                    return None;
                }

                let example_span = extract_first_function_parameter(ctx, call.span()).map_or_else(
                    || "$span".to_string(),
                    |param| format!("(metadata ${param}).span"),
                );

                Some(
                    Violation::new(
                        "error make is missing 'labels' field for error location",
                        call.span(),
                    )
                    .with_primary_label("missing labels")
                    .with_help(format!(
                        "Add a 'labels' field to pinpoint where the error occurred:\nlabels: {{ \
                         text: \"describe the problem\", span: {example_span} }}"
                    )),
                )
            })
            .into_iter()
            .collect()
    })
}

pub const RULE: Rule = Rule::new(
    "add_label_to_error",
    "error make should include 'labels' field with span to show error location in user code",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/error_make.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
