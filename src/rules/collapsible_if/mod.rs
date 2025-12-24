use nu_protocol::ast::{Call, Expr};

use crate::{
    Fix, LintLevel, Replacement, ast::call::CallExt, context::LintContext, rule::Rule,
    violation::Violation,
};

/// Creates a violation with fix for a collapsible if statement
fn create_violation(call: &Call, inner_call: &Call, fix_text: String) -> Violation {
    let fix = Fix::with_explanation(
        "Collapse nested if statements",
        vec![Replacement::new(call.span(), fix_text)],
    );

    Violation::new(
        "Nested if statement can be collapsed using 'and'",
        call.span(),
    )
    .with_primary_label("outer if")
    .with_extra_label(
        "inner if can be merged with outer condition",
        inner_call.span(),
    )
    .with_help("Combine conditions using 'and' instead of nesting if statements")
    .with_fix(fix)
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) if call.is_call_to_command("if", ctx) => {
            let Some(inner_call) = call.get_nested_single_if(ctx) else {
                return vec![];
            };
            call.generate_collapsed_if(ctx)
                .map(|fix_text| create_violation(call, inner_call, fix_text))
                .into_iter()
                .collect()
        }
        _ => vec![],
    })
}

pub const RULE: Rule = Rule::new(
    "collapsible_if",
    "Collapse nested if statements without else clauses into a single if with combined conditions",
    check,
    LintLevel::Warning,
)
.with_auto_fix()
.with_doc_url("https://www.nushell.sh/book/control_flow.html");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
