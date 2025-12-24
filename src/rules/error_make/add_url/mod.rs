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
                if !has_field(record, "msg", ctx) || has_field(record, "url", ctx) {
                    return None;
                }

                Some(
                    Violation::new("error make is missing 'url' field for documentation link", call.span())
                        .with_primary_label("missing url")
                        .with_help(
                            "Add a 'url' field linking to relevant documentation:\n\
                             url: \"https://example.com/docs/error-explanation\"",
                        ),
                )
            })
            .into_iter()
            .collect()
    })
}

pub const RULE: Rule = Rule::new(
    "add_url_to_error",
    "error make should include 'url' field to link to documentation",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/commands/docs/error_make.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
