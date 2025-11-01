use nu_protocol::ast::{Expr, PipelineElement};

use crate::{
    ast::{CallExt, ExpressionExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const ERROR_INDICATORS: &[&str] = &[
    "error",
    "failed",
    "cannot",
    "unable",
    "invalid",
    "not found",
    "missing",
    "denied",
    "forbidden",
    "unauthorized",
    "timeout",
    "connection",
    "network",
    "unreachable",
];

fn looks_like_error(message: &str, exit_code: i64) -> bool {
    exit_code != 0
        && ERROR_INDICATORS
            .iter()
            .any(|indicator| message.to_lowercase().contains(indicator))
}

/// Extract the message from a print command call
fn extract_print_message(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<String> {
    // Get the first positional argument (the message)
    let message_expr = call.get_first_positional_arg()?;

    // Extract string content
    match &message_expr.expr {
        Expr::String(s) => Some(s.clone()),
        _ => Some(message_expr.span_text(context).to_string()),
    }
}

/// Extract the exit code from an exit command call
fn extract_exit_code(call: &nu_protocol::ast::Call) -> Option<i64> {
    // Get the first positional argument (the exit code)
    let code_expr = call.get_first_positional_arg()?;

    match &code_expr.expr {
        Expr::Int(code) => Some(*code),
        _ => None,
    }
}

/// Check if two pipeline elements are sequential print and exit calls
fn check_sequential_print_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<RuleViolation> {
    // Check if first element is a print call
    let Expr::Call(print_call) = &first.expr.expr else {
        return None;
    };

    if !print_call.is_call_to_command("print", context) {
        return None;
    }

    // Check if second element is an exit call
    let Expr::Call(exit_call) = &second.expr.expr else {
        return None;
    };

    if !exit_call.is_call_to_command("exit", context) {
        return None;
    }

    // Extract message and exit code
    let message = extract_print_message(print_call, context)?;
    let exit_code = extract_exit_code(exit_call)?;

    // Check if it looks like an error
    looks_like_error(&message, exit_code).then(|| {
        RuleViolation::new_static(
            "prefer_error_make",
            "Consider using 'error make' instead of 'print' + 'exit' for error conditions",
            print_call.span().merge(exit_call.span()),
        )
        .with_suggestion_static(
            "Use 'error make { msg: \"error message\" }' for better error handling",
        )
    })
}

/// Check consecutive pipelines in a block for print + exit pattern
fn check_block_pipelines(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    for pipelines in block.pipelines.windows(2) {
        if let [first_pipeline, second_pipeline] = pipelines {
            // Each pipeline should have exactly one element for this pattern
            if first_pipeline.elements.len() != 1 || second_pipeline.elements.len() != 1 {
                continue;
            }

            if let Some(violation) = check_sequential_print_exit(
                &first_pipeline.elements[0],
                &second_pipeline.elements[0],
                context,
            ) {
                violations.push(violation);
            }
        }
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Check the main block
    check_block_pipelines(context.ast, context, &mut violations);

    // Recursively check all nested blocks (closures, functions, etc.)
    // Note: We only check Block/Closure expressions here because if/while/for
    // command blocks are already traversed by collect_rule_violations
    violations.extend(
        context.collect_rule_violations(|expr, ctx| match &expr.expr {
            Expr::Closure(block_id) | Expr::Block(block_id) => {
                let mut nested_violations = Vec::new();
                let block = ctx.working_set.get_block(*block_id);
                check_block_pipelines(block, ctx, &mut nested_violations);
                nested_violations
            }
            _ => vec![],
        }),
    );

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_error_make",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Use 'error make' for custom errors instead of 'print' + 'exit'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
