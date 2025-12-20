use nu_protocol::ast::{Expr, Expression};

use crate::{
    ast::call::CallExt,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

fn is_newline_string(expr: &Expression, context: &LintContext) -> bool {
    let text = context.get_span_text(expr.span);
    // Only double-quoted strings interpret \n as newline in Nushell
    // Single-quoted strings are raw and treat \n literally
    matches!(text, "\"\\n\"" | "\"\\r\\n\"")
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        if !call.is_call_to_command("split row", ctx) {
            return vec![];
        }

        let Some(separator_arg) = call.get_first_positional_arg() else {
            return vec![];
        };

        if !is_newline_string(separator_arg, ctx) {
            return vec![];
        }

        let fix = Fix::with_explanation(
            "Replace with 'lines'",
            vec![Replacement::new(expr.span, "lines")],
        );

        vec![
            Violation::new(
                "Use 'lines' instead of 'split row \"\\n\"' for splitting by newlines",
                expr.span,
            )
            .with_primary_label("inefficient newline split")
            .with_help(
                "The 'lines' command is more efficient and clearer for splitting text by newlines",
            )
            .with_fix(fix),
        ]
    })
}
pub const fn rule() -> Rule {
    Rule::new(
        "prefer_lines_over_split",
        "Use 'lines' instead of 'split row \"\\n\"' for better performance and clarity",
        check,
    )
    .with_doc_url("https://www.nushell.sh/commands/docs/lines.html")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
