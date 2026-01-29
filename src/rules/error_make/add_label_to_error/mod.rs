use nu_protocol::ast::Expr;

use super::{extract_record_from_expr, has_field};
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

struct AddLabelToError;

impl DetectFix for AddLabelToError {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "add_label_to_error"
    }

    fn short_description(&self) -> &'static str {
        "error make should include 'label'"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Errors created with `error make` are more useful if you attach a label (with a span) \
             to signify the part of the code that caused the error.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/error_make.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
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

            if !has_field(record, "msg", ctx)
                || has_field(record, "labels", ctx)
                || has_field(record, "label", ctx)
            {
                return vec![];
            }

            vec![
                Detection::from_global_span(
                    "error make is missing 'label' (or 'labels') field for error location",
                    call.span(),
                )
                .with_primary_label("error missing label"),
            ]
        }))
    }
}

pub static RULE: &dyn Rule = &AddLabelToError;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
