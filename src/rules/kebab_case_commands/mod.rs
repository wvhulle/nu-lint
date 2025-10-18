use heck::ToKebabCase;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{AstRule, RuleCategory, RuleMetadata},
    visitor::{AstVisitor, VisitContext},
};

#[derive(Default)]
pub struct KebabCaseCommands;

impl KebabCaseCommands {
    /// Check if a command name follows kebab-case convention
    fn is_valid_kebab_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Allow single characters
        if name.len() == 1 {
            return name.chars().all(|c| c.is_ascii_lowercase());
        }

        // Check kebab-case pattern: lowercase letters, numbers, and hyphens
        // Must start with lowercase letter
        // Cannot have consecutive hyphens
        name.chars().enumerate().all(|(i, c)| {
            match c {
                'a'..='z' | '0'..='9' => true,
                '-' => {
                    // Cannot start with hyphen
                    if i == 0 {
                        return false;
                    }
                    // Cannot have consecutive hyphens
                    name.chars().nth(i + 1) != Some('-')
                }
                _ => false,
            }
        }) && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
    }
}

impl RuleMetadata for KebabCaseCommands {
    fn id(&self) -> &'static str {
        "kebab_case_commands"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Custom commands should use kebab-case naming convention"
    }
}

impl AstRule for KebabCaseCommands {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = KebabCaseCommandsVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that checks command naming using AST traversal
pub struct KebabCaseCommandsVisitor<'a> {
    rule: &'a KebabCaseCommands,
    violations: Vec<Violation>,
}

impl<'a> KebabCaseCommandsVisitor<'a> {
    #[must_use]
    pub fn new(rule: &'a KebabCaseCommands) -> Self {
        Self {
            rule,
            violations: Vec::new(),
        }
    }

    fn check_command_name(&mut self, cmd_name: &str, span: nu_protocol::Span) {
        if !KebabCaseCommands::is_valid_kebab_case(cmd_name) {
            use crate::lint::{Fix, Replacement};

            let kebab_case_name = cmd_name.to_kebab_case();

            let fix = Some(Fix {
                description: format!("Rename command '{cmd_name}' to '{kebab_case_name}'"),
                replacements: vec![Replacement {
                    span,
                    new_text: kebab_case_name.clone(),
                }],
            });

            self.violations.push(Violation {
                rule_id: self.rule.id().to_string(),
                severity: self.rule.severity(),
                message: format!(
                    "Command '{cmd_name}' should use kebab-case naming convention"
                ),
                span,
                suggestion: Some(format!(
                    "Consider renaming to: {kebab_case_name}"
                )),
                fix,
                file: None,
            });
        }
    }
}

impl AstVisitor for KebabCaseCommandsVisitor<'_> {
    fn visit_call(&mut self, call: &nu_protocol::ast::Call, context: &VisitContext) {
        // Check for def commands (function definitions)
        let decl = context.get_decl(call.decl_id);
        if decl.name() == "def" || decl.name() == "export def" {
            // The first argument to def should be the command name
            if let Some(first_arg) = call.arguments.first()
                && let nu_protocol::ast::Argument::Positional(expr) = first_arg {
                    // Extract command name from the span
                    let cmd_name = context.get_span_contents(expr.span);
                    self.check_command_name(cmd_name, expr.span);
                }
        }

        // Continue walking
        crate::visitor::walk_call(self, call, context);
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
