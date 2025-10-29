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
        "grep",
        BuiltinAlternative::with_note(
            "where or find",
            "Use 'where $it =~ <pattern>' for regex filtering, 'find <substring>' for text search",
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

    let (replacement, description) = if args_text.len() == 1 && !args_text[0].starts_with('-') {
        (
            format!("find \"{}\"", args_text[0]),
            "Use 'find' which is case-insensitive by default and works on structured data"
                .to_string(),
        )
    } else if args_text.contains(&"-i".to_string()) {
        (
            "where $it =~ \"pattern\"".to_string(),
            "Use 'find' (case-insensitive by default) or 'where $it =~ pattern' for regex \
             filtering. The -i flag is redundant in Nu"
                .to_string(),
        )
    } else {
        (
            "where $it =~ \"pattern\"".to_string(),
            "Use 'where $it =~ pattern' for regex filtering or 'find' for simple text search"
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
        "prefer_builtin_grep",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_grep",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'find' or 'where' instead of 'grep' for better data handling",
        check,
    )
}

#[cfg(test)]
mod tests;
