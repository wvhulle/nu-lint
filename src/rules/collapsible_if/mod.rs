use nu_protocol::ast::{Call, Expr};

use crate::{
    Fix, LintLevel, Replacement, ast::call::CallExt, context::LintContext, rule::Rule,
    violation::Violation,
};

/// Creates a violation with fix for a collapsible if statement
fn create_violation(call: &Call, fix_text: String) -> Violation {
    let fix = Fix::new_static(
        "Collapse nested if statements",
        vec![Replacement::new_dynamic(call.span(), fix_text)],
    );

    Violation::new_static(
        "collapsible_if",
        "Nested if statement can be collapsed using 'and'",
        call.span(),
    )
    .with_suggestion_static("Combine conditions using 'and' instead of nesting if statements")
    .with_fix(fix)
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) if call.is_call_to_command("if", ctx) => call
            .generate_collapsed_if(ctx)
            .map(|fix_text| create_violation(call, fix_text))
            .into_iter()
            .collect(),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "collapsible_if",
        LintLevel::Warn,
        "Collapse nested if statements without else clauses into a single if with combined \
         conditions",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
