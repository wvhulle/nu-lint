use heck::ToSnakeCase;
use nu_protocol::ast::{Argument, Expr};

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    rules::naming::NuNaming,
    violation::{Fix, Replacement, RuleViolation, Severity},
};

/// Check if this is a let or mut declaration
fn get_var_decl_type(decl_name: &str) -> Option<bool> {
    match decl_name {
        "let" => Some(false),
        "mut" => Some(true),
        _ => None,
    }
}

/// Create a violation for invalid variable name
fn create_snake_case_violation(
    var_name: &str,
    is_mutable: bool,
    name_span: nu_protocol::Span,
) -> RuleViolation {
    let var_type = if is_mutable {
        "Mutable variable"
    } else {
        "Variable"
    };
    let snake_case_name = var_name.to_snake_case();

    let fix = Fix {
        description: format!("Rename variable '{var_name}' to '{snake_case_name}'").into(),
        replacements: vec![Replacement {
            span: name_span,
            new_text: snake_case_name.clone().into(),
        }],
    };

    RuleViolation::new_dynamic(
        "snake_case_variables",
        format!("{var_type} '{var_name}' should use snake_case naming convention"),
        name_span,
    )
    .with_suggestion_dynamic(format!("Consider renaming to: {snake_case_name}"))
    .with_fix(fix)
}

/// Check a single call expression for variable naming violations
fn check_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Option<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    let is_mutable = get_var_decl_type(decl.name())?;

    let Argument::Positional(name_expr) = call.arguments.first()? else {
        return None;
    };

    let var_name = ctx.source.get(name_expr.span.start..name_expr.span.end)?;

    (!var_name.is_valid_snake_case())
        .then(|| create_snake_case_violation(var_name, is_mutable, name_expr.span))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        check_call(call, ctx).into_iter().collect()
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "snake_case_variables",
        RuleCategory::Naming,
        Severity::Info,
        "Variables should use snake_case naming convention",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
