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
    map.insert("ls", BuiltinAlternative::simple("ls"));
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

    let replacement = if args_text.is_empty() {
        "ls".to_string()
    } else {
        format!("ls {}", args_text.join(" "))
    };

    let description = "Use Nu's built-in 'ls' which returns structured table data with name, \
                       type, size, and modified columns";

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
        "prefer_builtin_ls",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_ls",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's built-in 'ls' instead of external ls command for structured data",
        check,
    )
}

#[cfg(test)]
mod tests;
