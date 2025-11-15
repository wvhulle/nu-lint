use heck::ToKebabCase;
use nu_protocol::ast::Expr;

use super::NuNaming;
use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        let decl_name = call.get_call_name(ctx);
        if !matches!(decl_name.as_str(), "def" | "export def") {
            return vec![];
        }

        let Some((cmd_name, name_span)) = call.extract_declaration_name(ctx) else {
            return vec![];
        };

        if cmd_name.is_valid_kebab_case() {
            return vec![];
        }

        vec![cmd_name.create_naming_violation(
            "kebab_case_commands",
            "Command",
            &cmd_name.to_kebab_case(),
            name_span,
        )]
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "kebab_case_commands",
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
