use nu_protocol::ast::Expr;

use super::{extract_record_from_expr, has_field};
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
                if !has_field(record, "msg", ctx) || has_field(record, "help", ctx) {
                    return None;
                }

                Some(
                    Violation::new(
                        "error make is missing 'help' field to guide users",
                        call.span(),
                    )
                    .with_primary_label("missing help")
                    .with_help(
                        "Add a 'help' field explaining how to fix the issue:\nhelp: \"describe \
                         how to resolve this error\"",
                    ),
                )
            })
            .into_iter()
            .collect()
    })
}

pub const RULE: Rule = Rule::new(
    "add_help_to_error",
    "error make should include 'help' field to guide users toward a solution",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/error_make.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
