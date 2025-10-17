//! Shared utilities for rules that detect external commands with builtin
//! alternatives

use std::{collections::HashMap, fmt::Write};

use nu_protocol::ast::Expr;

// Re-export Fix type for use by fix builders
pub use crate::lint::Fix;
use crate::{
    lint::{Severity, Violation},
    visitor::{AstVisitor, VisitContext},
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
    context: &VisitContext,
) -> Fix;

/// Generic AST visitor for detecting external commands with builtin
/// alternatives
pub struct ExternalCommandVisitor<'a> {
    rule_id: &'a str,
    severity: Severity,
    violations: Vec<Violation>,
    alternatives: HashMap<&'static str, BuiltinAlternative>,
    fix_builder: Option<FixBuilder>,
}

impl<'a> ExternalCommandVisitor<'a> {
    #[must_use]
    pub fn new(
        rule_id: &'a str,
        severity: Severity,
        alternatives: HashMap<&'static str, BuiltinAlternative>,
        fix_builder: Option<FixBuilder>,
    ) -> Self {
        Self {
            rule_id,
            severity,
            violations: Vec::new(),
            alternatives,
            fix_builder,
        }
    }

    #[must_use]
    pub fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    /// Check for special command usage patterns that need custom suggestions
    fn get_custom_suggestion(
        &self,
        cmd_text: &str,
        args: &[nu_protocol::ast::ExternalArgument],
        context: &VisitContext,
    ) -> Option<(String, String)> {
        match cmd_text {
            "tail" => {
                let args_text = context.extract_external_args(args);
                if args_text.iter().any(|arg| arg == "--pid") {
                    // Special case for tail --pid - this is process monitoring
                    let message = "Consider using Nushell's structured approach for process \
                                   monitoring instead of external 'tail --pid'"
                        .to_string();
                    let suggestion = "Replace 'tail --pid $pid -f /dev/null' with Nushell process \
                                      monitoring:\nwhile (ps | where pid == $pid | length) > 0 { \
                                      sleep 1s }\n\nThis approach uses Nushell's built-in ps \
                                      command with structured data filtering and is more portable \
                                      across platforms."
                        .to_string();
                    return Some((message, suggestion));
                }
            }
            "hostname" => {
                let args_text = context.extract_external_args(args);
                if args_text.iter().any(|arg| arg == "-I") {
                    // Special case for hostname -I - this gets IP addresses, not hostname
                    let message = "Consider using Nushell's structured approach for getting IP \
                                   addresses instead of external 'hostname -I'"
                        .to_string();
                    let suggestion = "Replace 'hostname -I' with Nushell network commands:\nsys \
                                      net | get ip\n\nThis approach uses Nushell's built-in sys \
                                      net command to get IP addresses in a structured format. You \
                                      can filter specific interfaces or addresses as needed."
                        .to_string();
                    return Some((message, suggestion));
                }
            }
            _ => {}
        }
        None
    }
}

impl AstVisitor for ExternalCommandVisitor<'_> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        // Check for external calls
        if let Expr::ExternalCall(head, args) = &expr.expr {
            // Get the command name from the head expression
            let cmd_text = context.get_span_contents(head.span);

            // Check for custom suggestions first
            if let Some((custom_message, custom_suggestion)) =
                self.get_custom_suggestion(cmd_text, args, context)
            {
                self.violations.push(Violation {
                    rule_id: self.rule_id.to_string(),
                    severity: self.severity,
                    message: custom_message,
                    span: expr.span,
                    suggestion: Some(custom_suggestion),
                    fix: None, // Custom suggestions don't have automatic fixes yet
                    file: None,
                });
                return;
            }

            // Check if this external command has a builtin alternative
            if let Some(alternative) = self.alternatives.get(cmd_text) {
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

                // Build fix if a fix builder is provided
                let fix = self
                    .fix_builder
                    .map(|builder| builder(cmd_text, alternative, args, expr.span, context));

                self.violations.push(Violation {
                    rule_id: self.rule_id.to_string(),
                    severity: self.severity,
                    message,
                    span: expr.span,
                    suggestion: Some(suggestion),
                    fix,
                    file: None,
                });
            }
        }

        crate::visitor::walk_expression(self, expr, context);
    }
}
