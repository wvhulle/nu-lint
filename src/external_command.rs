use std::collections::HashMap;

use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    violation::{Fix, RuleViolation},
};

/// Extract external command arguments as strings
#[must_use]
pub(crate) fn extract_external_args(
    args: &[nu_protocol::ast::ExternalArgument],
    context: &LintContext,
) -> Vec<String> {
    args.iter()
        .map(|arg| match arg {
            nu_protocol::ast::ExternalArgument::Regular(expr) => {
                context.source[expr.span.start..expr.span.end].to_string()
            }
            nu_protocol::ast::ExternalArgument::Spread(expr) => {
                format!("...{}", &context.source[expr.span.start..expr.span.end])
            }
        })
        .collect()
}

/// Metadata about a builtin alternative to an external command
pub(crate) struct BuiltinAlternative {
    pub command: &'static str,
    pub note: Option<&'static str>,
}

impl BuiltinAlternative {
    #[must_use]
    pub fn simple(command: &'static str) -> Self {
        Self {
            command,
            note: None,
        }
    }

    #[must_use]
    pub fn with_note(command: &'static str, note: &'static str) -> Self {
        Self {
            command,
            note: Some(note),
        }
    }
}

/// Type alias for a function that builds a fix for a specific external command
pub(crate) type FixBuilder = fn(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix;

/// Detect external commands with builtin alternatives
#[must_use]
pub(crate) fn detect_external_commands<S: ::std::hash::BuildHasher>(
    context: &LintContext,
    rule_id: &'static str,
    alternatives: &HashMap<&'static str, BuiltinAlternative, S>,
    fix_builder: Option<FixBuilder>,
) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = &ctx.source[head.span.start..head.span.end];

            if let Some(alternative) = alternatives.get(cmd_text) {
                let message = format!(
                    "Consider using Nushell's built-in '{}' instead of external '^{}'",
                    alternative.command, cmd_text
                );

                let suggestion = match alternative.note {
                    Some(note) => format!(
                        "Replace '^{}' with built-in command: {}\nBuilt-in commands are more \
                         portable, faster, and provide better error handling.\n\nNote: {note}",
                        cmd_text, alternative.command
                    ),
                    None => format!(
                        "Replace '^{}' with built-in command: {}\nBuilt-in commands are more \
                         portable, faster, and provide better error handling.",
                        cmd_text, alternative.command
                    ),
                };

                let fix =
                    fix_builder.map(|builder| builder(cmd_text, alternative, args, expr.span, ctx));

                let violation = RuleViolation::new_dynamic(rule_id, message, expr.span)
                    .with_suggestion_dynamic(suggestion);

                let violation = match fix {
                    Some(f) => violation.with_fix(f),
                    None => violation,
                };

                return vec![violation];
            }
        }
        vec![]
    })
}
