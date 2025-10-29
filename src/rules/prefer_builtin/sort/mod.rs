use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix},
    lint::{Replacement, Severity},
    rule::{Rule, RuleCategory},
};

fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();
    map.insert("sort", BuiltinAlternative::simple("sort or sort-by"));
    map
}

fn build_fix(
    _cmd_text: &str,
    _alternative: &BuiltinAlternative,
    _args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    _context: &LintContext,
) -> Fix {
    let description = "Use Nu's built-in 'sort' which works on any data type and supports natural \
                       sorting with -n flag";

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: "sort".into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_builtin_sort",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_sort",
        RuleCategory::Idioms,
        Severity::Info,
        "Use Nu's 'sort' command for better data type support",
        check,
    )
}

#[cfg(test)]
mod tests;
