use heck::ToKebabCase;
use nu_protocol::ast::Expr;

use super::NuNaming;
use crate::{
    ast_utils::{CallExt, DeclarationUtils},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Check a single call expression for command naming violations
fn check_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Option<RuleViolation> {
    let decl_name = call.get_call_name(ctx);

    if !DeclarationUtils::is_def_command(&decl_name) {
        return None;
    }

    let (cmd_name, name_span) = DeclarationUtils::extract_declaration_name(call, ctx)?;

    if !cmd_name.is_valid_kebab_case() {
        let kebab_case_name = cmd_name.to_kebab_case();
        return Some(cmd_name.create_naming_violation(
            "kebab_case_commands",
            "Command",
            &kebab_case_name,
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
