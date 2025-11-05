use std::collections::HashMap;

use nu_protocol::ast::Expr;

pub use crate::violation::Fix;
use crate::{context::LintContext, violation::RuleViolation};

/// Extract external command arguments as strings
#[must_use]
pub fn extract_external_args(
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
pub struct BuiltinAlternative {
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
pub type FixBuilder = fn(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix;

/// Check for special command usage patterns that need custom suggestions
fn get_custom_suggestion(
    cmd_text: &str,
    args: &[nu_protocol::ast::ExternalArgument],
    context: &LintContext,
) -> Option<(String, String)> {
    match cmd_text {
        "tail" => {
            let args_text = extract_external_args(args, context);
            if args_text.iter().any(|arg| arg == "--pid") {
                let message = "Consider using Nushell's structured approach for process \
                               monitoring instead of external 'tail --pid'"
                    .to_string();
                let suggestion = "Replace 'tail --pid $pid -f /dev/null' with Nushell process \
                                  monitoring:\nwhile (ps | where pid == $pid | length) > 0 { \
                                  sleep 1s }\n\nThis approach uses Nushell's built-in ps command \
                                  with structured data filtering and is more portable across \
                                  platforms."
                    .to_string();
                return Some((message, suggestion));
            }
        }
        "hostname" => {
            let args_text = extract_external_args(args, context);
            if args_text.iter().any(|arg| arg == "-I") {
                let message = "Consider using Nushell's structured approach for getting IP \
                               addresses instead of external 'hostname -I'"
                    .to_string();
                let suggestion = "Replace 'hostname -I' with Nushell network commands:\nsys net | \
                                  get ip\n\nThis approach uses Nushell's built-in sys net command \
                                  to get IP addresses in a structured format. You can filter \
                                  specific interfaces or addresses as needed."
                    .to_string();
                return Some((message, suggestion));
            }
        }
        _ => {}
    }
    None
}

/// Detect external commands with builtin alternatives
#[must_use]
pub fn detect_external_commands<S: ::std::hash::BuildHasher>(
    context: &LintContext,
    rule_id: &'static str,
    alternatives: &HashMap<&'static str, BuiltinAlternative, S>,
    fix_builder: Option<FixBuilder>,
) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = &ctx.source[head.span.start..head.span.end];

            if let Some((custom_message, custom_suggestion)) =
                get_custom_suggestion(cmd_text, args, ctx)
            {
                return vec![
                    RuleViolation::new_dynamic(rule_id, custom_message, expr.span)
                        .with_suggestion_dynamic(custom_suggestion),
                ];
            }

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
