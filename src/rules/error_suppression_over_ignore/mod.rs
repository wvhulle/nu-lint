use nu_protocol::ast::Expr;

use crate::{
    ast::CallExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Destructive operations that users often mistakenly try to silence with `|
/// ignore`
///
/// This includes:
/// - File operations: rm, mv, cp, mkdir, touch, mktemp
/// - Database operations: stor delete, stor insert, stor update
/// - Network operations: http delete, http post, http put, http patch
const DESTRUCTIVE_OPERATIONS: &[&str] = &[
    // Filesystem operations
    "rm",
    "mv",
    "cp",
    "mkdir",
    "touch",
    "mktemp",
    // Database operations
    "stor delete",
    "stor insert",
    "stor update",
    // Network operations (modify remote state)
    "http delete",
    "http post",
    "http put",
    "http patch",
];

/// Check if a call is a destructive operation
fn is_destructive_file_operation(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    let cmd_name = call.get_call_name(context);

    // Check if it's in our list of always-destructive operations
    if DESTRUCTIVE_OPERATIONS.contains(&cmd_name.as_str()) {
        return true;
    }

    // Special case: save is only destructive with -f/--force flag
    if cmd_name == "save" {
        return call.arguments.iter().any(|arg| {
            matches!(arg, nu_protocol::ast::Argument::Named(named)
                if named.0.item == "force")
        });
    }

    false
}

/// Recursively check if an expression contains a file operation call
fn contains_file_operation_in_expr(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> bool {
    match &expr.expr {
        Expr::Call(call) => {
            is_destructive_file_operation(call, context)
                || call.arguments.iter().any(|arg| {
                    matches!(
                        arg,
                        nu_protocol::ast::Argument::Positional(e)
                            if contains_file_operation_in_expr(e, context)
                    )
                })
        }
        Expr::Block(block_id) | Expr::Closure(block_id) => context
            .working_set
            .get_block(*block_id)
            .pipelines
            .iter()
            .any(|pipeline| {
                pipeline
                    .elements
                    .iter()
                    .any(|elem| contains_file_operation_in_expr(&elem.expr, context))
            }),
        _ => false,
    }
}

/// Check if a pipeline ends with `| ignore` and contains file operations
fn check_pipeline(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    let last_elem = (pipeline.elements.len() >= 2).then(|| pipeline.elements.last())??;

    let Expr::Call(last_call) = &last_elem.expr.expr else {
        return None;
    };

    (last_call.get_call_name(context) == "ignore").then_some(())?;

    let has_file_operation = pipeline
        .elements
        .iter()
        .take(pipeline.elements.len() - 1)
        .any(|elem| contains_file_operation_in_expr(&elem.expr, context));

    has_file_operation.then_some(())?;

    let pipeline_start = pipeline.elements.first()?.expr.span.start;
    let ignore_start = last_elem.expr.span.start;
    let command_text = context.source[pipeline_start..ignore_start]
        .trim()
        .trim_end_matches('|')
        .trim();

    let suggestion = format!(
        "'| ignore' only discards output, not errors. For error suppression:\n\nInstead of:  \
         {command_text} | ignore\nUse:         do -i {{ {command_text} }}\n\nOr use try-catch for \
         explicit error handling:\ntry {{ {command_text} }} catch {{ print 'failed' }}"
    );

    Some(
        RuleViolation::new_static(
            "prefer_error_suppression_over_ignore",
            "Using '| ignore' with file operations doesn't suppress errors - use 'do -i { ... }' \
             instead",
            last_elem.expr.span,
        )
        .with_suggestion_dynamic(suggestion),
    )
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context
        .ast
        .pipelines
        .iter()
        .filter_map(|pipeline| check_pipeline(pipeline, context))
        .chain(context.collect_rule_violations(|expr, ctx| {
            match &expr.expr {
                Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                    ctx.working_set
                        .get_block(*block_id)
                        .pipelines
                        .iter()
                        .filter_map(|pipeline| check_pipeline(pipeline, ctx))
                        .collect()
                }
                _ => vec![],
            }
        }))
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_error_suppression_over_ignore",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "File operations with '| ignore' don't suppress errors - use 'do -i { ... }' instead",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
