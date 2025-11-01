use heck::ToKebabCase;
use nu_protocol::ast::Expr;

use crate::{
    ast_utils::{CallExt, DeclarationUtils, NamingUtils},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
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

/// Check a single call expression for command naming violations
fn check_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Option<RuleViolation> {
    let decl_name = call.get_call_name(ctx);

    if !DeclarationUtils::is_def_command(&decl_name) {
        return None;
    }

    let (cmd_name, name_span) = DeclarationUtils::extract_declaration_name(call, ctx)?;

    if !is_valid_kebab_case(&cmd_name) {
        let kebab_case_name = cmd_name.to_kebab_case();
        return Some(NamingUtils::create_naming_violation(
            "kebab_case_commands",
            "Command",
            &cmd_name,
            kebab_case_name,
            name_span,
        ));
    }

    None
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
