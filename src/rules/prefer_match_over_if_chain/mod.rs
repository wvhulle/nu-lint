use nu_protocol::ast::{Expr, Operator};

use crate::{
    ast::{CallExt, ExpressionExt, SpanExt},
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
    // Extract all branches: (condition_value, body_text)
    let mut branches: Vec<(String, String)> = Vec::new();
    let mut current_call = call;
    let mut has_final_else = false;
    let mut final_else_body = String::new();

    loop {
        // Get condition and extract the compared value
        let condition_arg = current_call.get_first_positional_arg()?;
        let compared_value = extract_compared_value(condition_arg, context)?;

        // Get the then-body (2nd positional argument)
        let then_arg = current_call.get_positional_arg(1)?;
        let then_body = then_arg.span_text(context).trim().to_string();

        branches.push((compared_value, then_body));

        // Check for else branch
        let Some(else_arg) = current_call.get_positional_arg(2) else {
            // No else branch at all
            log::debug!("No else branch found");
            break;
        };

        log::debug!(
            "Else arg expr type: {:?}",
            std::mem::discriminant(&else_arg.expr)
        );

        match &else_arg.expr {
            Expr::Keyword(keyword) => {
                // This could be either an else-if chain continuation or a final else
                log::debug!("Found Keyword, checking if it's an else-if");
                match &keyword.expr.expr {
                    Expr::Call(next_call) if next_call.is_call_to_command("if", context) => {
                        log::debug!("It's an else-if, continuing chain");
                        current_call = next_call;
                    }
                    Expr::Block(_block_id) => {
                        // This is a final else block wrapped in a Keyword
                        log::debug!("Found final else block wrapped in Keyword");
                        has_final_else = true;
                        final_else_body = keyword.expr.span_text(context).trim().to_string();
                        break;
                    }
                    _ => {
                        log::debug!("Keyword doesn't contain an if call or block");
                        break;
                    }
                }
            }
            Expr::Block(_block_id) => {
                // This is a final else block (not wrapped in Keyword)
                log::debug!("Found final else block");
                has_final_else = true;
                final_else_body = else_arg.span_text(context).trim().to_string();
                break;
            }
            _ => {
                log::debug!("Unexpected else arg type");
                break;
            }
        }
    }

    log::debug!("has_final_else: {has_final_else}, final_else_body: {final_else_body}");
    log::debug!("Number of branches: {}", branches.len());

    // Build the match expression
    // var_name already includes the $ prefix
    let mut match_text = format!("match {var_name} {{\n");

    for (value, body) in &branches {
        // Format the match arm
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

/// Extract the value being compared from a comparison expression like $var ==
/// "value"
fn extract_compared_value(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<String> {
    let Expr::BinaryOp(_left, _op, right) = &expr.expr else {
        return None;
    };

    Some(right.span_text(context).to_string())
}

/// Analyze an if-call and its else branch to detect if-else-if chains
#[allow(clippy::too_many_lines)]
fn analyze_if_chain(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<RuleViolation> {
    log::debug!("Analyzing if call at span {:?}", call.head);
    log::debug!("Call has {} arguments", call.arguments.len());

    // Get the condition expression
    let condition_arg = call.get_first_positional_arg()?;
    log::debug!(
        "Got first positional arg (condition): {:?}",
        condition_arg.span
    );

    // Check if this is comparing a variable (e.g., $var == value)
    let compared_var = extract_compared_variable(condition_arg, context);
    log::debug!("Extracted compared variable: {compared_var:?}");
    let compared_var = compared_var?;

    // Get the else argument (3rd positional for if command)
    log::debug!("Looking for else argument at position 2");
    let else_arg = call.get_positional_arg(2);
    log::debug!("Else argument: {:?}", else_arg.map(|a| &a.expr));
    let else_arg = else_arg?;

    // The else argument can be either:
    // 1. A Block (final else { ... })
    // 2. A Keyword containing an if Call (else if ...)
    let (is_else_if, nested_call) = match &else_arg.expr {
        Expr::Keyword(keyword) => {
            // This is an else-if chain
            log::debug!("Else is a Keyword, checking inner expression");
            match &keyword.expr.expr {
                Expr::Call(call) if call.is_call_to_command("if", context) => {
                    log::debug!("Found else-if!");
                    (true, call)
                }
                _ => {
                    log::debug!("Keyword doesn't contain an if call");
                    return None;
                }
            }
        }
        Expr::Block(_) => {
            log::debug!("Else is a Block (final else), not an else-if chain");
            return None;
        }
        _ => {
            log::debug!("Else argument is neither Keyword nor Block");
            return None;
        }
    };

    if !is_else_if {
        return None;
    }

    log::debug!("Found if-else-if chain! Starting chain walk...");

    // This is an if-else-if chain! Now count how many branches
    let mut chain_length = 2; // We have at least 2 (the outer if and one else-if)
    let current_var = compared_var.clone();
    let mut current_call = nested_call;
    let mut all_same_var = true;

    // Walk the chain
    loop {
        let Some(condition_arg) = current_call.get_first_positional_arg() else {
            break;
        };

        if let Some(var) = extract_compared_variable(condition_arg, context) {
            if var != current_var {
                all_same_var = false;
            }
        } else {
            all_same_var = false;
        }

        // Check for another else-if
        let Some(else_arg) = current_call.get_positional_arg(2) else {
            break;
        };

        // Check if this is another else-if (Keyword) or final else (Block)
        let next_call = match &else_arg.expr {
            Expr::Keyword(keyword) => {
                // This is an else-if chain continuation
                match &keyword.expr.expr {
                    Expr::Call(call) if call.is_call_to_command("if", context) => call,
                    _ => break,
                }
            }
            Expr::Block(_) => {
                // This is a final else, not another else-if
                break;
            }
            _ => break,
        };

        chain_length += 1;
        current_call = next_call;
    }

    // Only flag if we have 3+ branches (at least 2 else-if)
    if chain_length >= 3 {
        let fix = if all_same_var {
            build_match_fix(call, &current_var, context)
        } else {
            None
        };

        let violation = if all_same_var {
            RuleViolation::new_dynamic(
                "prefer_match_over_if_chain",
                format!(
                    "If-else-if chain comparing '{current_var}' to different values - consider \
                     using 'match'"
                ),
                call.span(),
            )
            .with_suggestion_dynamic(
                "Use 'match $var { value1 => { ... }, value2 => { ... }, _ => { ... } }' for \
                 clearer value-based branching"
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
    } else {
        None
    }
}

/// Extract the variable name from a comparison expression like $var == value
fn extract_compared_variable(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<String> {
    log::debug!(
        "extract_compared_variable: expr type = {:?}",
        std::mem::discriminant(&expr.expr)
    );
    let Expr::BinaryOp(left, op, _right) = &expr.expr else {
        log::debug!("Not a BinaryOp");
        return None;
    };

    log::debug!(
        "BinaryOp left type = {:?}",
        std::mem::discriminant(&left.expr)
    );

    // Check if it's an equality comparison
    let Expr::Operator(Operator::Comparison(
        nu_protocol::ast::Comparison::Equal | nu_protocol::ast::Comparison::NotEqual,
    )) = &op.expr
    else {
        log::debug!("Not an equality comparison operator");
        return None;
    };

    // Extract variable from left side - try proper extraction first
    if let Some(var_name) = left.extract_variable_name(context) {
        log::debug!("Extracted variable name from left: {var_name:?}");
        return Some(var_name);
    }

    // Fallback: if the left side looks like a variable reference (FullCellPath),
    // extract the text even if the variable isn't properly declared
    if let Expr::FullCellPath(cell_path) = &left.expr {
        let text = cell_path.head.span.text(context);
        log::debug!("Fallback: extracted text from FullCellPath: {text}");
        Some(text.to_string())
    } else {
        log::debug!("Left side is not a FullCellPath either");
        None
    }
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
