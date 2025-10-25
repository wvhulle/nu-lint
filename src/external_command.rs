use std::{collections::HashMap, fmt::Write};

use nu_protocol::ast::Expr;

// Re-export Fix type for use by fix builders
pub use crate::lint::Fix;
use crate::{
    context::LintContext,
    lint::{Severity, Violation},
};

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

/// Helper function to extract external command arguments as strings
fn extract_external_args(
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

/// Detect external commands with builtin alternatives
pub fn detect_external_commands(
    context: &LintContext,
    rule_id: &str,
    severity: Severity,
    alternatives: &HashMap<&'static str, BuiltinAlternative>,
    fix_builder: Option<FixBuilder>,
) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| {
        if let Expr::ExternalCall(head, args) = &expr.expr {
            let cmd_text = &ctx.source[head.span.start..head.span.end];

            // Check for custom suggestions first
            if let Some((custom_message, custom_suggestion)) =
                get_custom_suggestion(cmd_text, args, ctx)
            {
                return vec![Violation {
                    rule_id: rule_id.to_string().into(),
                    severity,
                    message: custom_message.into(),
                    span: expr.span,
                    suggestion: Some(custom_suggestion.into()),
                    fix: None,
                    file: None,
                }];
            }

            // Check if this external command has a builtin alternative
            if let Some(alternative) = alternatives.get(cmd_text) {
                let message = format!(
                    "Consider using Nushell's built-in '{}' instead of external '^{}'",
                    alternative.command, cmd_text
                );

                let mut suggestion = format!(
                    "Replace '^{}' with built-in command: {}\nBuilt-in commands are more \
                     portable, faster, and provide better error handling.",
                    cmd_text, alternative.command
                );

                if let Some(note) = alternative.note {
                    write!(suggestion, "\n\nNote: {note}").unwrap();
                }

                let fix =
                    fix_builder.map(|builder| builder(cmd_text, alternative, args, expr.span, ctx));

                return vec![Violation {
                    rule_id: rule_id.to_string().into(),
                    severity,
                    message: message.into(),
                    span: expr.span,
                    suggestion: Some(suggestion.into()),
                    fix,
                    file: None,
                }];
            }
        }
        vec![]
    })
}
