use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline},
};

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Whitelist of external commands that are generally safe and unlikely to fail
/// These commands typically only fail if the system is severely broken
const SAFE_EXTERNAL_COMMANDS: &[&str] = &[
    // Basic shell utilities that rarely fail
    "echo", "printf", "true", "false", "yes", "seq", "ls", // Date/time commands
    "date", "uptime", "cal", // Information display commands (read-only, safe)
    "whoami", "id", "hostname", "uname", "arch", // Path commands
    "pwd", "basename", "dirname", "realpath", "readlink", // Environment
    "env", "printenv", // Simple text processing (no file I/O)
    "tr", "cut", "paste", "column", "fmt", "fold", "expand", "unexpand", // Math
    "bc", "dc", "expr", // Safe directory operations
    "mktemp", "git",
];

fn is_alias_or_export_definition(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline
        .elements
        .first()
        .and_then(|element| {
            if let Expr::Call(call) = &element.expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                Some(matches!(
                    decl_name,
                    "alias"
                        | "export"
                        | "export alias"
                        | "export def"
                        | "export const"
                        | "export use"
                        | "export extern"
                        | "def"
                        | "const"
                ))
            } else {
                None
            }
        })
        .unwrap_or(false)
}

/// Check if an external command is dangerous (likely to fail)
/// Returns the command name if it's dangerous, None otherwise
fn get_dangerous_external_command(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<String> {
    use nu_protocol::ast::Traverse;

    let mut commands = Vec::new();
    expr.flat_map(
        context.working_set,
        &|e| {
            if let Expr::ExternalCall(head, _args) = &e.expr {
                let head_text = context.source[head.span.start..head.span.end].to_string();
                if is_safe_command(&head_text) {
                    vec![]
                } else {
                    vec![head_text]
                }
            } else {
                vec![]
            }
        },
        &mut commands,
    );

    commands.into_iter().next()
}

/// Whitelist of commands that are generally safe and unlikely to fail
/// These commands typically only fail if the system is severely broken
fn is_safe_command(cmd: &str) -> bool {
    SAFE_EXTERNAL_COMMANDS.contains(&cmd)
}

fn check_pipeline_for_external_commands(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    // Single element pipelines don't have the issue - the exit code is checked
    if pipeline.elements.len() <= 1 {
        return None;
    }

    // Check if wrapped in error handling (try, complete, or do -i)
    if is_pipeline_wrapped_in_error_handling(pipeline, context) {
        return None;
    }

    // Look for dangerous external commands that are NOT in the last position
    // Only the last command's exit code is checked by Nushell
    let last_idx = pipeline.elements.len() - 1;

    pipeline
        .elements
        .iter()
        .enumerate()
        .take(last_idx) // Skip last element
        .find_map(|(_, element)| {
            get_dangerous_external_command(&element.expr, context)
                .map(|_cmd| create_violation(element.expr.span, pipeline, context))
        })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    for pipeline in &block.pipelines {
        // Check the pipeline itself unless it's a definition
        if !is_alias_or_export_definition(pipeline, context) {
            violations.extend(check_pipeline_for_external_commands(pipeline, context));
        }

        // Always check for nested blocks (functions, closures, etc.)
        check_pipeline_for_nested_blocks(pipeline, context, violations);
    }
}

fn check_pipeline_for_nested_blocks(
    pipeline: &Pipeline,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    use nu_protocol::ast::Traverse;

    for element in &pipeline.elements {
        let mut blocks = Vec::new();
        element.expr.flat_map(
            context.working_set,
            &|expr| match &expr.expr {
                Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                    vec![*block_id]
                }
                _ => vec![],
            },
            &mut blocks,
        );

        for &block_id in &blocks {
            let block = context.working_set.get_block(block_id);
            check_block(block, context, violations);
        }
    }
}

fn is_in_try_block(expr_span: Span, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    let mut try_spans = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            matches!(&expr.expr, Expr::Call(call)
            if context.working_set.get_decl(call.decl_id).name() == "try")
            .then_some(expr.span)
            .into_iter()
            .collect()
        },
        &mut try_spans,
    );

    try_spans
        .iter()
        .any(|try_span| try_span.contains_span(expr_span))
}

fn pipeline_has_complete(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if context.working_set.get_decl(call.decl_id).name() == "complete")
    })
}

fn pipeline_has_do_ignore(pipeline: &Pipeline, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;

    pipeline.elements.iter().any(|element| {
        let mut found = Vec::new();
        element.expr.flat_map(
            context.working_set,
            &|expr| {
                if is_do_with_ignore_flag(expr, context) {
                    vec![()]
                } else {
                    vec![]
                }
            },
            &mut found,
        );
        !found.is_empty()
    })
}

fn is_do_with_ignore_flag(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    let Expr::Call(call) = &expr.expr else {
        return false;
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    decl_name == "do" && has_ignore_errors_flag(call)
}

fn has_ignore_errors_flag(call: &nu_protocol::ast::Call) -> bool {
    call.arguments.iter().any(|arg| {
        matches!(arg, nu_protocol::ast::Argument::Named(named)
            if named.0.item == "ignore_errors" || named.0.item == "i")
    })
}

fn is_pipeline_wrapped_in_error_handling(pipeline: &Pipeline, context: &LintContext) -> bool {
    // Check if any element is in a try block
    pipeline.elements.iter().any(|element| is_in_try_block(element.expr.span, context))
        // Check if pipeline uses complete
        || pipeline_has_complete(pipeline, context)
        // Check if pipeline uses do -i
        || pipeline_has_do_ignore(pipeline, context)
}

fn create_violation(span: Span, _pipeline: &Pipeline, _context: &LintContext) -> RuleViolation {
    let message = "External command in pipeline without error handling: Nushell only checks the \
                   last command's exit code. If this command fails, the error will be silently \
                   ignored.";

    let suggestion = "Handle errors from pipeline commands:\n\n\
        1. Use 'try' block (recommended for simple cases):\n\
           try {\n\
               ^curl https://api.example.com | from json\n\
           }\n\n\
        2. Use 'complete' with exit code checking (for custom error handling):\n\
           let result = (^curl https://api.example.com | complete)\n\
           if $result.exit_code != 0 {\n\
               error make { msg: $\"Command failed: ($result.stderr)\" }\n\
           }\n\
           $result.stdout | from json\n\n\
        3. Use 'do -i' to explicitly ignore errors (when errors can be safely ignored):\n\
           do -i {\n\
               ^curl https://api.example.com | from json\n\
           }";

    RuleViolation::new_static("pipeline_handle_errors", message, span)
        .with_suggestion_static(suggestion)
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    check_block(context.ast, context, &mut violations);
    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "pipeline_handle_errors",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Ensure external commands in pipelines have proper error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
