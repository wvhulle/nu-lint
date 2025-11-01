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
    call.get_first_positional_arg()
        .map(|message_expr| match &message_expr.expr {
            Expr::String(s) => s.clone(),
            _ => message_expr.span_text(context).to_string(),
        })
}

/// Extract the exit code from an exit command call
fn extract_exit_code(call: &nu_protocol::ast::Call) -> Option<i64> {
    call.get_first_positional_arg()
        .and_then(|code_expr| match &code_expr.expr {
            Expr::Int(code) => Some(*code),
            _ => None,
        })
}

/// Check if two pipeline elements are sequential print and exit calls
fn check_sequential_print_exit(
    first: &PipelineElement,
    second: &PipelineElement,
    context: &LintContext,
) -> Option<RuleViolation> {
    let print_call = match &first.expr.expr {
        Expr::Call(call) if call.is_call_to_command("print", context) => call,
        _ => return None,
    };

    let exit_call = match &second.expr.expr {
        Expr::Call(call) if call.is_call_to_command("exit", context) => call,
        _ => return None,
    };

    extract_print_message(print_call, context)
        .zip(extract_exit_code(exit_call))
        .filter(|(message, exit_code)| looks_like_error(message, *exit_code))
        .map(|_| {
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
fn check_block_pipelines<'a>(
    block: &'a nu_protocol::ast::Block,
    context: &'a LintContext<'a>,
) -> impl Iterator<Item = RuleViolation> + 'a {
    block.pipelines.windows(2).filter_map(move |pipelines| {
        let [first_pipeline, second_pipeline] = pipelines else {
            return None;
        };

        // Each pipeline should have exactly one element for this pattern
        let [first_elem] = &first_pipeline.elements[..] else {
            return None;
        };
        let [second_elem] = &second_pipeline.elements[..] else {
            return None;
        };

        check_sequential_print_exit(first_elem, second_elem, context)
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let main_violations = check_block_pipelines(context.ast, context);

    let nested_violations = context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Closure(block_id) | Expr::Block(block_id) => {
            let block = ctx.working_set.get_block(*block_id);
            check_block_pipelines(block, ctx).collect()
        }
        _ => vec![],
    });

    main_violations.chain(nested_violations).collect()
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
