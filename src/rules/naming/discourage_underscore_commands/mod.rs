use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    for (_decl_id, decl) in context.new_user_functions() {
        let command_name = &decl.signature().name;

        // Check if command name contains underscores
        if command_name.contains('_') {
            let suggested_name = command_name.replace('_', "-");
            let span = context.find_declaration_span(command_name);

            violations.push(
                RuleViolation::new_dynamic(
                    "discourage_underscore_commands",
                    format!(
                        "Command '{command_name}' uses underscores - prefer hyphens for \
                         readability"
                    ),
                    span,
                )
                .with_suggestion_dynamic(format!(
                    "Rename to '{suggested_name}' following Nushell convention"
                )),
            );
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
