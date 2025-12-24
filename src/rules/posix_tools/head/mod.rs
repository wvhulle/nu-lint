use nu_protocol::ast::ExternalArgument;

use crate::{
    LintLevel, Violation,
    alternatives::{detect_external_commands, external_args_slices},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

const NOTE: &str = "Use 'first N' to get the first N items";

fn build_fix(
    _cmd_text: &str,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let replacement = external_args_slices(args, context)
        .find(|a| a.starts_with('-') && a.len() > 1)
        .map_or_else(
            || "first 10".to_string(),
            |num_arg| {
                let num = &num_arg[1..];
                format!("first {num}")
            },
        );

    let description = "Use 'first' with cleaner syntax: 'first N' instead of 'head -N'";

    Fix {
        explanation: description.into(),
        replacements: vec![Replacement {
            span: expr_span.into(),
            replacement_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(context, "head", NOTE, Some(build_fix))
}

pub const RULE: Rule = Rule::new(
    "use_builtin_head",
    "Use Nu's 'first' command instead of 'head' for cleaner syntax",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/commands/docs/first.html");

#[cfg(test)]
mod tests;
