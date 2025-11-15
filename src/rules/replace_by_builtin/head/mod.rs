use std::collections::HashMap;

use nu_protocol::ast::ExternalArgument;

use crate::{
    Violation,
    ast::ext_command::{BuiltinAlternative, ExternalArgumentExt, detect_external_commands},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "head",
        BuiltinAlternative::with_note("first", "Use 'first N' to get the first N items"),
    );
    map
}

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = args.extract_as_strings(context);

    let replacement = args_text
        .iter()
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
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    detect_external_commands(
        context,
        "prefer_builtin_head",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_head",
        "Use Nu's 'first' command instead of 'head' for cleaner syntax",
        check,
    )
}

#[cfg(test)]
mod tests;
