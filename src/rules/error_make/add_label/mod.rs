use nu_protocol::ast::Expr;

use super::{extract_first_function_parameter, extract_record_from_expr, has_field};
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct AddLabelToError;

impl DetectFix for AddLabelToError {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "add_label_to_error"
    }

    fn explanation(&self) -> &'static str {
        "error make should include 'labels' field with span to show error location in user code"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/error_make.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(context.detect(|expr, ctx| {
            let Expr::Call(call) = &expr.expr else {
                return vec![];
            };

            if !call.is_call_to_command("error make", ctx) {
                return vec![];
            }

            let Some(record) = call
                .get_first_positional_arg()
                .and_then(extract_record_from_expr)
            else {
                return vec![];
            };

            if !has_field(record, "msg", ctx) || has_field(record, "labels", ctx) {
                return vec![];
            }

            let example_span = extract_first_function_parameter(ctx, call.span()).map_or_else(
                || "$span".to_string(),
                |param| format!("(metadata ${param}).span"),
            );

            vec![
                Detection::from_global_span(
                    "error make is missing 'labels' field for error location",
                    call.span(),
                )
                .with_primary_label("missing labels")
                .with_help(format!(
                    "Add a 'labels' field to pinpoint where the error occurred:\nlabels: {{ text: \
                     \"describe the problem\", span: {example_span} }}"
                )),
            ]
        }))
    }
}

pub static RULE: &dyn Rule = &AddLabelToError;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
