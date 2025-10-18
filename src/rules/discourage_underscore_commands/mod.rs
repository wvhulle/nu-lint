use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (_decl_id, decl) in context.new_user_functions() {
        let command_name = &decl.signature().name;

        // Check if command name contains underscores
        if command_name.contains('_') {
            let suggested_name = command_name.replace('_', "-");
            let span = context.find_declaration_span(command_name);

            violations.push(Violation {
                rule_id: "discourage_underscore_commands".into(),
                severity: Severity::Info,
                message: format!(
                    "Command '{command_name}' uses underscores - prefer hyphens for readability"
                )
                .into(),
                span,
                suggestion: Some(
                    format!("Rename to '{suggested_name}' following Nushell convention").into(),
                ),
                fix: None,
                file: None,
            });
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "discourage_underscore_commands",
        RuleCategory::Naming,
        Severity::Info,
        "Command names should use hyphens instead of underscores for better readability",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
