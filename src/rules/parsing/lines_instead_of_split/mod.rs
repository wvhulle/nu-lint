use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

pub struct FixData {
    replace_span: Span,
}

fn is_newline_string(expr: &Expression, context: &LintContext) -> bool {
    let text = context.plain_text(expr.span);
    // Only double-quoted strings interpret \n as newline in Nushell
    // Single-quoted strings are raw and treat \n literally
    matches!(text, "\"\\n\"" | "\"\\r\\n\"")
}

struct LinesInsteadOfSplit;

impl DetectFix for LinesInsteadOfSplit {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "lines_instead_of_split"
    }

    fn short_description(&self) -> &'static str {
        r#"Use 'lines' instead of 'split row "\n"' for better performance and clarity"#
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/lines.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
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

            let detected = Detection::from_global_span(
                "Use 'lines' instead of 'split row \"\\n\"' for splitting by newlines",
                expr.span,
            )
            .with_primary_label("inefficient newline split");

            let fix_data = FixData {
                replace_span: expr.span,
            };

            vec![(detected, fix_data)]
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix::with_explanation(
            "Replace with 'lines'",
            vec![Replacement::new(fix_data.replace_span, "lines")],
        ))
    }
}

pub static RULE: &dyn Rule = &LinesInsteadOfSplit;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
