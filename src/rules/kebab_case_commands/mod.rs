use heck::ToKebabCase;
use nu_protocol::ast::{Argument, Expr};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
};

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

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Call(call) => {
                // Check for def commands (function definitions)
                let decl = ctx.working_set.get_decl(call.decl_id);
                if decl.name() == "def" || decl.name() == "export def" {
                    // The first argument to def should be the command name
                    if let Some(Argument::Positional(name_expr)) = call.arguments.first() {
                        let cmd_name = ctx
                            .source
                            .get(name_expr.span.start..name_expr.span.end)
                            .unwrap_or("");

                        if !is_valid_kebab_case(cmd_name) {
                            let kebab_case_name = cmd_name.to_kebab_case();
                            let fix = Some(Fix {
                                description: format!(
                                    "Rename command '{cmd_name}' to '{kebab_case_name}'"
                                )
                                .into(),
                                replacements: vec![Replacement {
                                    span: name_expr.span,
                                    new_text: kebab_case_name.clone().into(),
                                }],
                            });

                            return vec![Violation {
                                rule_id: "kebab_case_commands".into(),
                                severity: Severity::Warning,
                                message: format!(
                                    "Command '{cmd_name}' should use kebab-case naming convention"
                                )
                                .into(),
                                span: name_expr.span,
                                suggestion: Some(
                                    format!("Consider renaming to: {kebab_case_name}").into(),
                                ),
                                fix,
                                file: None,
                            }];
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "kebab_case_commands",
        RuleCategory::Naming,
        Severity::Warning,
        "Custom commands should use kebab-case naming convention",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
