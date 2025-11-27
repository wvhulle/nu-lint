use nu_protocol::ast::{Call, Expr};

use crate::{
    Fix, Replacement, ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation,
};

/// Creates a violation with fix for a collapsible if statement
fn create_violation(call: &Call, fix_text: String) -> Violation {
    let fix = Fix::with_explanation(
        "Collapse nested if statements",
        vec![Replacement::new(call.span(), fix_text)],
    );

    Violation::new("Nested if statement can be collapsed using 'and'",
        call.span(),
    )
    .with_help("Combine conditions using 'and' instead of nesting if statements")
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

pub const fn rule() -> Rule {
    Rule::new(
        "collapsible_if",
        "Collapse nested if statements without else clauses into a single if with combined \
         conditions",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/control_flow.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
