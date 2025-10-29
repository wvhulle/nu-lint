use heck::ToKebabCase;
use nu_protocol::ast::{Argument, Expr};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, RuleViolation, Severity},
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

/// Check if this is a def or export def command
fn is_def_command(decl_name: &str) -> bool {
    matches!(decl_name, "def" | "export def")
}

/// Create a violation for invalid command name
fn create_kebab_case_violation(cmd_name: &str, name_span: nu_protocol::Span) -> RuleViolation {
    let kebab_case_name = cmd_name.to_kebab_case();

    let fix = Fix {
        description: format!("Rename command '{cmd_name}' to '{kebab_case_name}'").into(),
        replacements: vec![Replacement {
            span: name_span,
            new_text: kebab_case_name.clone().into(),
        }],
    };

    RuleViolation::new_dynamic(
        "kebab_case_commands",
        format!("Command '{cmd_name}' should use kebab-case naming convention"),
        name_span,
    )
    .with_suggestion_dynamic(format!("Consider renaming to: {kebab_case_name}"))
    .with_fix(fix)
}

/// Check a single call expression for command naming violations
fn check_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Option<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);

    is_def_command(decl.name()).then_some(())?;

    let Argument::Positional(name_expr) = call.arguments.first()? else {
        return None;
    };

    let cmd_name = ctx.source.get(name_expr.span.start..name_expr.span.end)?;

    (!is_valid_kebab_case(cmd_name)).then(|| create_kebab_case_violation(cmd_name, name_expr.span))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            check_call(call, ctx).into_iter().collect()
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "kebab_case_commands",
        RuleCategory::Naming,
        Severity::Info,
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
