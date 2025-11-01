use nu_protocol::ast::Expr;

use crate::{
    ast::{CallExt, ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

/// Build a fix that converts an if-else-if chain to a match expression
fn build_match_fix(
    call: &nu_protocol::ast::Call,
    var_name: &str,
    context: &LintContext,
) -> Option<Fix> {
    let mut branches: Vec<(String, String)> = Vec::new();
    let mut current_call = call;
    let mut has_final_else = false;
    let mut final_else_body = String::new();

    loop {
        // Get condition and extract the compared value
        let condition_arg = current_call.get_first_positional_arg()?;
        let compared_value = condition_arg.extract_comparison_value(context)?;

        // Get the then-body (2nd positional argument)
        let then_arg = current_call.get_positional_arg(1)?;
        let then_body = then_arg.span_text(context).trim().to_string();

        branches.push((compared_value, then_body));

        // Check for else branch
        let Some((is_else_if, else_expr)) = current_call.get_else_branch() else {
            break;
        };

        if is_else_if {
            // Continue with the next if call
            if let Expr::Call(next_call) = &else_expr.expr {
                current_call = next_call;
            } else {
                break;
            }
        } else {
            // Final else block
            has_final_else = true;
            final_else_body = else_expr.span_text(context).trim().to_string();
            break;
        }
    }

    // Build the match expression
    // var_name already includes the $ prefix
    let mut match_text = format!("match {var_name} {{\n");

    for (value, body) in &branches {
        use std::fmt::Write;
        writeln!(&mut match_text, "    {value} => {body},").unwrap();
    }

    if has_final_else {
        use std::fmt::Write;
        writeln!(&mut match_text, "    _ => {final_else_body}").unwrap();
    }

    match_text.push('}');

    Some(Fix::new_dynamic(
        format!("Convert to match expression on {var_name}"),
        vec![Replacement::new_dynamic(call.span(), match_text)],
    ))
}

/// Analyze an if-call and its else branch to detect if-else-if chains
fn analyze_if_chain(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<RuleViolation> {
    // Get the condition expression and check if it compares a variable
    let condition_arg = call.get_first_positional_arg()?;
    let compared_var = condition_arg.extract_compared_variable(context)?;

    // Check if this has an else-if branch (not just a final else)
    let (is_else_if, else_expr) = call.get_else_branch()?;
    if !is_else_if {
        return None;
    }

    // Get the nested if call from the else-if
    let Expr::Call(nested_call) = &else_expr.expr else {
        return None;
    };

    if !nested_call.is_call_to_command("if", context) {
        return None;
    }

    // Walk the chain to count branches and check if all compare the same variable
    let mut chain_length = 2; // We have at least 2 (outer if + one else-if)
    let mut all_same_var = true;
    let mut current_call = nested_call;

    loop {
        // Check if this branch compares the same variable
        if let Some(condition_arg) = current_call.get_first_positional_arg() {
            let same_var = condition_arg
                .extract_compared_variable(context)
                .is_some_and(|var| var == compared_var);

            if !same_var {
                all_same_var = false;
            }
        }

        // Check for another else-if
        let Some((is_else_if, else_expr)) = current_call.get_else_branch() else {
            break;
        };

        if !is_else_if {
            break;
        }

        let Expr::Call(next_call) = &else_expr.expr else {
            break;
        };

        if !next_call.is_call_to_command("if", context) {
            break;
        }

        chain_length += 1;
        current_call = next_call;
    }

    // Only flag if we have 3+ branches (at least 2 else-if)
    if chain_length < 3 {
        return None;
    }

    let fix = if all_same_var {
        build_match_fix(call, &compared_var, context)
    } else {
        None
    };

    let violation = if all_same_var {
        RuleViolation::new_dynamic(
            "prefer_match_over_if_chain",
            format!(
                "If-else-if chain comparing '{compared_var}' to different values - consider using \
                 'match'"
            ),
            call.span(),
        )
        .with_suggestion_dynamic(
            "Use 'match $var { value1 => { ... }, value2 => { ... }, _ => { ... } }' for clearer \
             value-based branching"
                .to_string(),
        )
    } else {
        RuleViolation::new_static(
            "prefer_match_over_if_chain",
            "Long if-else-if chain - consider using 'match' for clearer branching",
            call.span(),
        )
        .with_suggestion_static(
            "For multiple related conditions, 'match' provides clearer pattern matching",
        )
    };

    Some(if let Some(fix) = fix {
        violation.with_fix(fix)
    } else {
        violation
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr
            && call.is_call_to_command("if", ctx)
        {
            return analyze_if_chain(call, ctx).into_iter().collect();
        }
        vec![]
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_match_over_if_chain",
        RuleCategory::Idioms,
        Severity::Warning,
        "Use 'match' for value-based branching instead of if-else-if chains",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
