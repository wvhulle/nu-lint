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
}

impl AstVisitor for ExternalCommandVisitor<'_> {
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
