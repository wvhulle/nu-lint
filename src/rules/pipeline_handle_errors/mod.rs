use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline},
};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn is_alias_or_export_definition(pipeline: &Pipeline, context: &LintContext) -> bool {
    pipeline
        .elements
        .first()
        .and_then(|element| {
            if let Expr::Call(call) = &element.expr.expr {
                let decl_name = context.working_set.get_decl(call.decl_id).name();
                log::debug!("Pipeline first element is a Call to: {decl_name}");
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
                log::debug!("Pipeline first element is NOT a Call");
                None
            }
        })
        .unwrap_or(false)
}

/// Check if a pipeline contains external commands that are not in error-handling wrappers
/// Returns a violation if found
fn check_pipeline_for_external_commands(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    log::debug!("Checking pipeline with {} elements", pipeline.elements.len());
    
    // Single element pipelines don't have the issue - the exit code is checked
    if pipeline.elements.len() <= 1 {
        log::debug!("Pipeline has only 1 element, skipping");
        return None;
    }

    // Check if wrapped in try or complete or do -i
    if is_pipeline_wrapped_in_error_handling(pipeline, context) {
        log::debug!("Pipeline is wrapped in error handling");
        return None;
    }

    // Look for external commands that are NOT in the last position
    // Check the TOP LEVEL expression of each pipeline element
    let last_idx = pipeline.elements.len() - 1;
    
    for (idx, element) in pipeline.elements.iter().enumerate() {
        if idx >= last_idx {
            // Last element is OK - its exit code is checked
            continue;
        }
        
        log::debug!("Checking pipeline element {idx} (top-level): {:?}", element.expr.expr);
        
        // Check if THIS element (or anything it contains) is an external call
        if contains_external_call(&element.expr, context) {
            log::debug!("Element {idx} contains external call - creating violation");
            return Some(create_violation(element.expr.span, pipeline, context));
        }
    }

    None
}

/// Check if an expression or any of its sub-expressions contains an external call
fn contains_external_call(expr: &nu_protocol::ast::Expression, context: &LintContext) -> bool {
    use nu_protocol::ast::Traverse;
    
    let mut results = Vec::new();
    expr.flat_map(
        context.working_set,
        &|e| {
            if let Expr::ExternalCall(head, _args) = &e.expr {
                let head_text = &context.source[head.span.start..head.span.end];
                log::debug!("    Found ExternalCall in traversal: {head_text:?}");
                
                // If it's an ExternalCall, the user explicitly used ^ prefix
                // This means they want to run the external version, regardless of
                // whether a builtin exists with the same name
                log::debug!("    Confirmed as external command (via ^ prefix): {head_text:?}");
                vec![true]
            } else {
                vec![]
            }
        },
        &mut results,
    );
    
    let has_external = !results.is_empty();
    log::debug!("    Contains external: {has_external}");
    has_external
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<RuleViolation>) {
    log::debug!("Checking block with {} pipelines", block.pipelines.len());

    // Check each pipeline for external commands that aren't in the last position
    block.pipelines.iter().for_each(|pipeline| {
        // Skip checking the pipeline itself if it's a definition, but still check nested blocks
        if is_alias_or_export_definition(pipeline, context) {
            log::debug!("Skipping - pipeline is an alias/export definition (but will check nested blocks)");
        } else if let Some(violation) = check_pipeline_for_external_commands(pipeline, context) {
            log::debug!("Creating violation for external command in pipeline");
            violations.push(violation);
        }

        // Always check for nested blocks (functions, closures, etc.)
        check_pipeline_for_nested_blocks(pipeline, context, violations);
    });
}

/// Check a pipeline for nested blocks and recursively check them
fn check_pipeline_for_nested_blocks(
    pipeline: &Pipeline,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    use nu_protocol::ast::Traverse;

    pipeline.elements.iter().for_each(|element| {
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
    });
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

    let mut has_do_ignore = false;
    
    for element in &pipeline.elements {
        let mut found = Vec::new();
        element.expr.flat_map(
            context.working_set,
            &|expr| check_expr_for_do_ignore(expr, context),
            &mut found,
        );
        if !found.is_empty() {
            has_do_ignore = true;
            break;
        }
    }

    has_do_ignore
}

fn check_expr_for_do_ignore(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Vec<bool> {
    if let Expr::Call(call) = &expr.expr {
        let decl_name = context.working_set.get_decl(call.decl_id).name();
        if decl_name != "do" {
            return vec![];
        }
        
        // Check if the call has the ignore_errors flag set
        // The `do` command has an `ignore_errors` named parameter
        let has_ignore_flag = call.arguments.iter().any(|arg| {
            // Check if this is a named argument with ignore_errors
            matches!(arg, nu_protocol::ast::Argument::Named(named) 
                if named.0.item == "ignore_errors" || named.0.item == "i")
        });
        
        if has_ignore_flag {
            vec![true]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn is_pipeline_wrapped_in_error_handling(pipeline: &Pipeline, context: &LintContext) -> bool {
    // Check if any element is in a try block
    for element in &pipeline.elements {
        if is_in_try_block(element.expr.span, context) {
            log::debug!("Pipeline element is in try block");
            return true;
        }
    }

    // Check if pipeline uses complete
    if pipeline_has_complete(pipeline, context) {
        log::debug!("Pipeline contains complete");
        return true;
    }

    // Check if pipeline uses do -i
    if pipeline_has_do_ignore(pipeline, context) {
        log::debug!("Pipeline uses do -i");
        return true;
    }

    false
}

fn create_violation(span: Span, _pipeline: &Pipeline, _context: &LintContext) -> RuleViolation {
    let message = "External command in pipeline without error handling: \
        Nushell only checks the last command's exit code. \
        If this command fails, the error will be silently ignored.";

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
        Severity::Warning,
        "Ensure external commands in pipelines have proper error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
