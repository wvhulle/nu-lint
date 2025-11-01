use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix, extract_external_args},
    rule::{Rule, RuleCategory},
    violation::{Replacement, Severity},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "tail",
        BuiltinAlternative::with_note("last", "Use 'last N' to get the last N items"),
    );
    map
}

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);

    let replacement =
        if let Some(num_arg) = args_text.iter().find(|a| a.starts_with('-') && a.len() > 1) {
            let num = &num_arg[1..];
            format!("last {num}")
        } else {
            "last 10".to_string()
        };

    let description = "Use 'last' with cleaner syntax: 'last N' instead of 'tail -N'";

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: replacement.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_builtin_tail",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_tail",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'last' command instead of 'tail' for cleaner syntax",
        check,
    )
}

#[cfg(test)]
mod tests;
