use nu_protocol::ast::Expr;

use super::{extract_record_from_expr, has_field};
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct AddUrlToError;

impl DetectFix for AddUrlToError {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "add_url_to_error"
    }

    fn explanation(&self) -> &'static str {
        "error make should include 'url' field to link to documentation"
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

            call.get_first_positional_arg()
                .and_then(extract_record_from_expr)
                .and_then(|record| {
                    if !has_field(record, "msg", ctx) || has_field(record, "url", ctx) {
                        return None;
                    }

                    Some(
                        Detection::from_global_span(
                            "error make is missing 'url' field for documentation link",
                            call.span(),
                        )
                        .with_primary_label("missing url")
                        .with_help(
                            "Add a 'url' field linking to relevant documentation:\n\
                             url: \"https://example.com/docs/error-explanation\"",
                        ),
                    )
                })
                .into_iter()
                .collect()
        }))
    }
}

pub static RULE: &dyn Rule = &AddUrlToError;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
