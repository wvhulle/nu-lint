use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

pub struct DiscourageUnderscoreCommands;

impl DiscourageUnderscoreCommands {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiscourageUnderscoreCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleMetadata for DiscourageUnderscoreCommands {
    fn id(&self) -> &'static str {
        "discourage_underscore_commands"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Command names should use hyphens instead of underscores for better readability"
    }
}

impl RegexRule for DiscourageUnderscoreCommands {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (_decl_id, decl) in context.new_user_functions() {
            let command_name = &decl.signature().name;

            // Check if command name contains underscores
            if command_name.contains('_') {
                let suggested_name = command_name.replace('_', "-");
                let span = context.find_declaration_span(command_name);

                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command '{command_name}' uses underscores - prefer hyphens for \
                         readability"
                    ),
                    span,
                    suggestion: Some(format!(
                        "Rename to '{suggested_name}' following Nushell convention"
                    )),
                    fix: None,
                    file: None,
                });
            }
        }

        violations
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
