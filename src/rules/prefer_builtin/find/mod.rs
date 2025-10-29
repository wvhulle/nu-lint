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
        "find",
        BuiltinAlternative::with_note(
            "ls or glob",
            "Use 'ls **/*.ext' for recursive file matching, 'glob **/*.ext' for pattern matching",
        ),
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

    let (replacement, description) = if args_text.iter().any(|arg| arg.contains("*.")) {
        let repl = if let Some(pattern) = args_text.iter().find(|arg| arg.contains("*.")) {
            format!("ls **/{}", pattern.trim_matches('"'))
        } else {
            "ls **/*".to_string()
        };
        (
            repl,
            "Use 'ls' with glob patterns (**/*.ext) for recursive file searches".to_string(),
        )
    } else if args_text.len() == 1 && !args_text[0].starts_with('-') {
        (
            format!("ls {}/**/*", args_text[0]),
            "Use 'ls' with glob patterns for directory traversal".to_string(),
        )
    } else {
        (
            "ls **/*".to_string(),
            "Use 'ls' with glob patterns for file finding, or 'glob' for more complex patterns"
                .to_string(),
        )
    };

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
        "prefer_builtin_find",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_find",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'ls' with glob patterns instead of 'find' command",
        check,
    )
}

#[cfg(test)]
mod tests;
