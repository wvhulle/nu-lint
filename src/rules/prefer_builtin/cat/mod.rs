use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix, extract_external_args},
    lint::{Replacement, Severity},
    rule::{Rule, RuleCategory},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert(
        "cat",
        BuiltinAlternative::with_note(
            "open --raw",
            "Use 'open' to read files as structured data, or 'open --raw' for plain text",
        ),
    );
    map
}

fn build_fix(
    _cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);

    let replacement = if let Some(file) = args_text.iter().find(|a| !a.starts_with('-')) {
        format!("open --raw {file}")
    } else {
        alternative.command.to_string()
    };

    let description = "Use 'open --raw' for plain text, or just 'open' to auto-parse structured \
                       files (JSON, TOML, CSV, etc.)";

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
        "prefer_builtin_cat",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_cat",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'open' command instead of 'cat' for better file handling",
        check,
    )
}

#[cfg(test)]
mod tests;
