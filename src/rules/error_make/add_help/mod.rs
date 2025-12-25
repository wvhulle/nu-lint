use nu_protocol::ast::{Expr, Expression};

use super::{extract_record_from_expr, has_field};
use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn check_error_make_call(expr: &Expression, ctx: &LintContext) -> Vec<Detection> {
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

    if !has_field(record, "msg", ctx) || has_field(record, "help", ctx) {
        return vec![];
    }

    vec![
        Detection::from_global_span(
            "error make is missing 'help' field to guide users",
            call.span(),
        )
        .with_primary_label("missing help")
        .with_help(
            "Add a 'help' field explaining how to fix the issue:\nhelp: \"describe how to resolve \
             this error\"",
        ),
    ]
}

struct AddHelpToError;

impl DetectFix for AddHelpToError {
    type FixInput = ();

    fn id(&self) -> &'static str {
        "add_help_to_error"
    }

    fn explanation(&self) -> &'static str {
        "error make should include 'help' field to guide users toward a solution"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/error_make.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect(&self, context: &LintContext) -> Vec<(Detection, Self::FixInput)> {
        Self::no_fix(context.detect(check_error_make_call))
    }
}

pub static RULE: &dyn Rule = &AddHelpToError;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
