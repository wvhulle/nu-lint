//! Shared utilities for rules that detect external commands with builtin alternatives

use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{Severity, Violation};
use nu_protocol::ast::Expr;
use std::collections::HashMap;

// Re-export Fix type for use by fix builders
pub use crate::context::Fix;

/// Metadata about a builtin alternative to an external command
pub struct BuiltinAlternative {
    pub command: &'static str,
    pub note: Option<&'static str>,
}

impl BuiltinAlternative {
    pub fn simple(command: &'static str) -> Self {
        Self {
            command,
            note: None,
        }
    }

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
) -> Option<Fix>;

/// Generic AST visitor for detecting external commands with builtin alternatives
pub struct ExternalCommandVisitor<'a> {
    rule_id: &'a str,
    severity: Severity,
    violations: Vec<Violation>,
    alternatives: HashMap<&'static str, BuiltinAlternative>,
    fix_builder: Option<FixBuilder>,
}

impl<'a> ExternalCommandVisitor<'a> {
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

    pub fn into_violations(self) -> Vec<Violation> {
        self.violations
    }
}

impl<'a> AstVisitor for ExternalCommandVisitor<'a> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        // Check for external calls
        if let Expr::ExternalCall(head, args) = &expr.expr {
            // Get the command name from the head expression
            let cmd_text = context.get_span_contents(head.span);

            // Check if this external command has a builtin alternative
            if let Some(alternative) = self.alternatives.get(cmd_text) {
                let message = format!(
                    "Consider using Nushell's built-in '{}' instead of external '^{}'",
                    alternative.command, cmd_text
                );

                let mut suggestion = format!(
                    "Replace '^{}' with built-in command: {}\n\
                     Built-in commands are more portable, faster, and provide better error handling.",
                    cmd_text,
                    alternative.command
                );

                if let Some(note) = alternative.note {
                    suggestion.push_str(&format!("\n\nNote: {}", note));
                }

                // Build fix if a fix builder is provided
                let fix = self
                    .fix_builder
                    .and_then(|builder| builder(cmd_text, alternative, args, expr.span, context));

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

        // Continue walking the AST
        crate::ast_walker::walk_expression(self, expr, context);
    }
}
